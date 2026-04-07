"""kubectl blocks — Kubernetes operations via a bound session.

Usage::

    with ssh.connect(ctx, container="my-app", host="10.0.0.1",
                     username="root", password=pw) as sesh:
        kube = kubectl.connect(sesh, namespace="prod")

        kube.get("deployment", name="web")
        kube.get_secret_value(secret="db-creds", key="password")
        pod = kube.find_pod(name_prefix="web")
        kube.exec(target="deploy/web", cmd="cat /app/config.py")
"""

from __future__ import annotations

import json as _json

from loguru import logger

from cutip_blocks.blocks.ssh import ExecResult, SSHSession


class KubectlSession:
    """Bound kubectl session over SSH with a default namespace.

    Created via ``kubectl.connect(sesh, namespace="prod")``.
    Every method runs a ``kubectl`` command over the bound SSH session.
    """

    def __init__(self, sesh: SSHSession, namespace: str) -> None:
        self._sesh = sesh
        self._namespace = namespace

    def _ns(self, override: str | None) -> str:
        return override or self._namespace

    def _run(self, cmd: str, *, block_name: str) -> ExecResult:
        return self._sesh.exec(f"kubectl {cmd}", block_name=block_name)

    def get(
        self,
        resource: str,
        *,
        name: str,
        namespace: str | None = None,
        output_format: str = "name",
    ) -> ExecResult:
        """Run ``kubectl get <resource> -n <namespace> <name> -o <output_format>``.

        Args:
            resource: K8s resource type (deployment, secret, pod, service, etc.).
            name: Resource name.
            namespace: Override the default namespace set in connect().
            output_format: kubectl output format (name, json, yaml, wide, jsonpath=...).

        Returns:
            ExecResult with stdout containing the kubectl output.

        Example::

            kube.get("deployment", name="web")
            kube.get("secret", name="my-creds", output_format="json")
        """
        ns = self._ns(namespace)
        result = self._run(
            f"get {resource} -n {ns} {name} -o {output_format}",
            block_name=f"kubectl get {resource}",
        )
        if result.exit_code != 0:
            raise RuntimeError(
                f"{resource} '{name}' not found in namespace {ns}: {result.stderr.strip()}"
            )
        return result

    def get_secret_value(self, *, secret: str, key: str, namespace: str | None = None) -> str:
        """Run ``kubectl get secret -n <ns> <secret> -o jsonpath="{.data.<key>}" | base64 -d``.

        Decodes a base64-encoded field from a Kubernetes secret.

        Args:
            secret: Secret resource name.
            key: Field name within the secret's data (e.g., "password", "token").
            namespace: Override the default namespace.

        Returns:
            The decoded secret value as a plain string.

        Example::

            password = kube.get_secret_value(secret="db-creds", key="password")
        """
        ns = self._ns(namespace)
        result = self._run(
            f'get secret -n {ns} {secret} -o jsonpath="{{.data.{key}}}" | base64 -d',
            block_name="kubectl get secret (decode)",
        )
        if result.exit_code != 0:
            raise RuntimeError(
                f"Failed to decode secret '{secret}' key '{key}' in {ns}: {result.stderr.strip()}"
            )
        return result.stdout.strip()

    def find_pod(self, *, name_prefix: str, namespace: str | None = None) -> str:
        """Find a pod by name prefix via ``kubectl get pod -n <ns> -o json``.

        Queries all pods as JSON, finds pods whose name starts with
        ``name_prefix`` (handles replica-set hash suffixes). Prefers Running pods.

        Args:
            name_prefix: Pod name prefix (typically the deployment name).
            namespace: Override the default namespace.

        Returns:
            The full pod name as a string.

        Example::

            pod = kube.find_pod(name_prefix="sfm-instance-rest-deployment")
            # Returns: "sfm-instance-rest-deployment-5bbbfc9575-sgd4v"
        """
        ns = self._ns(namespace)
        result = self._run(f"get pod -n {ns} -o json", block_name="kubectl find pod")
        if result.exit_code != 0:
            raise RuntimeError(f"Failed to list pods in namespace {ns}: {result.stderr.strip()}")

        try:
            data = _json.loads(result.stdout)
        except _json.JSONDecodeError:
            raise RuntimeError(f"Failed to parse pod list JSON from namespace {ns}")

        items = data.get("items", [])
        matches: list[dict] = []
        all_names: list[str] = []

        for pod in items:
            pod_name = pod.get("metadata", {}).get("name", "")
            all_names.append(pod_name)
            if pod_name.startswith(name_prefix):
                phase = pod.get("status", {}).get("phase", "Unknown")
                matches.append({"name": pod_name, "phase": phase})

        if not matches:
            available = ", ".join(all_names[:10]) or "(none)"
            raise RuntimeError(
                f"No pods matching '{name_prefix}' in namespace {ns}. Available pods: {available}"
            )

        running = [m for m in matches if m["phase"] == "Running"]
        selected = running[0] if running else matches[0]

        if len(matches) > 1:
            logger.warning(
                "[kubectl find pod] Multiple pods match '{}': {}. Using: {} ({})",
                name_prefix,
                ", ".join(f"{m['name']} ({m['phase']})" for m in matches),
                selected["name"],
                selected["phase"],
            )
        else:
            logger.info("[kubectl find pod] Found: {} ({})", selected["name"], selected["phase"])

        return selected["name"]

    def exec(self, *, target: str, cmd: str, namespace: str | None = None) -> ExecResult:
        """Run ``kubectl exec -n <namespace> <target> -- bash -c '<cmd>'``.

        Args:
            target: Pod or deployment target (e.g., ``"deploy/web"`` or ``"web-abc123"``).
            cmd: Shell command to execute inside the pod. Pipes are valid.
            namespace: Override the default namespace.

        Example::

            kube.exec(target="deploy/web", cmd="cat /app/config.py")
        """
        ns = self._ns(namespace)
        return self._run(
            f"exec -n {ns} {target} -- bash -c '{cmd}'",
            block_name="kubectl exec",
        )

    def cat_file(self, *, target: str, path: str, namespace: str | None = None) -> str:
        """Run ``kubectl exec -n <namespace> <target> -- cat <path>``.

        Reads a file from inside a pod without requiring ``tar`` (unlike ``kubectl cp``).

        Args:
            target: Pod or deployment target (e.g., ``"deploy/web"``).
            path: Absolute file path inside the pod.
            namespace: Override the default namespace.

        Returns:
            File content as a string.

        Example::

            content = kube.cat_file(target="deploy/web", path="/app/config.py")
        """
        ns = self._ns(namespace)
        result = self._run(
            f"exec -n {ns} {target} -- cat {path}",
            block_name="kubectl cat file",
        )
        if result.exit_code != 0:
            raise RuntimeError(f"Failed to read {path} from {target}: {result.stderr.strip()}")
        return result.stdout

    def cp(self, *, pod: str, src: str, dest: str, namespace: str | None = None) -> ExecResult:
        """Run ``kubectl cp <namespace>/<pod>:<src> <dest>``.

        Requires ``tar`` in the pod. If the pod lacks ``tar``, use ``cat_file()`` instead.

        Args:
            pod: Pod name (use ``find_pod()`` to resolve from deployment name).
            src: Source file path inside the pod.
            dest: Destination path on the remote filesystem.
            namespace: Override the default namespace.

        Example::

            pod = kube.find_pod(name_prefix="web")
            kube.cp(pod=pod, src="/app/data.json", dest="/tmp/data.json")
        """
        ns = self._ns(namespace)
        result = self._run(f"cp {ns}/{pod}:{src} {dest}", block_name="kubectl cp")
        if result.exit_code != 0:
            raise RuntimeError(f"kubectl cp failed: {result.stderr.strip()}")
        return result

    def apply(self, *, file: str) -> ExecResult:
        """Run ``kubectl apply -f <file>``.

        Args:
            file: Path to a YAML manifest on the remote filesystem.

        Example::

            kube.apply(file="/tmp/deployment-patch.yaml")
        """
        result = self._run(f"apply -f {file}", block_name="kubectl apply")
        if result.exit_code != 0:
            raise RuntimeError(f"kubectl apply failed: {result.stderr.strip()}")
        return result

    def rollout_status(
        self, *, resource: str, namespace: str | None = None, timeout: int = 120
    ) -> ExecResult:
        """Run ``kubectl rollout status <resource> -n <namespace> --timeout=<timeout>s``.

        Args:
            resource: Resource identifier (e.g., ``"deployment/web"``).
            namespace: Override the default namespace.
            timeout: Max wait time in seconds.

        Example::

            kube.rollout_status(resource="deployment/web", timeout=120)
        """
        ns = self._ns(namespace)
        result = self._run(
            f"rollout status {resource} -n {ns} --timeout={timeout}s",
            block_name="kubectl rollout status",
        )
        if result.exit_code != 0:
            raise RuntimeError(f"Rollout failed for {resource} in {ns}: {result.stderr.strip()}")
        logger.info("[kubectl rollout status] {} rolled out successfully", resource)
        return result

    def patch_deployment(
        self,
        *,
        deployment: str,
        volume_name: str,
        host_path: str,
        mount_path: str,
        namespace: str | None = None,
        rollout_timeout: int = 120,
    ) -> ExecResult:
        """Patch a deployment to add a hostPath volume + volumeMount, then apply and wait for rollout.

        Fetches the current deployment YAML, injects the volume/mount if not
        already present, strips metadata fields that block ``kubectl apply``,
        writes the patched YAML to a temp file on the remote host, applies it,
        and waits for the rollout to complete.

        Args:
            deployment: Deployment name.
            volume_name: Name for the volume and volumeMount.
            host_path: Host filesystem path to mount.
            mount_path: Mount path inside the container.
            namespace: Override the default namespace.
            rollout_timeout: Max seconds to wait for rollout (default 120).

        Example::

            kube.patch_deployment(
                deployment="web",
                volume_name="config-override",
                host_path="/root/patches/config.py",
                mount_path="/opt/app/config.py",
            )
        """
        import yaml as _yaml

        ns = self._ns(namespace)
        remote_patch_file = f"/tmp/{deployment}-patched.yaml"

        # Fetch current deployment YAML
        logger.info("[kubectl patch deployment] Fetching {}/{} ...", ns, deployment)
        result = self._run(
            f"get deployment -n {ns} {deployment} -o yaml", block_name="kubectl get deployment"
        )
        if result.exit_code != 0 or not result.stdout.strip():
            raise RuntimeError(
                f"Failed to get deployment {deployment} in {ns}: {result.stderr.strip()}"
            )

        doc = _yaml.safe_load(result.stdout)
        spec = doc.get("spec", {}).get("template", {}).get("spec", {})

        # Inject volume
        volume_entry = {"name": volume_name, "hostPath": {"path": host_path, "type": ""}}
        volumes = spec.get("volumes") or []
        if not any(v.get("name") == volume_name for v in volumes):
            volumes.append(volume_entry)
            spec["volumes"] = volumes
            logger.info("[kubectl patch deployment] Added volume '{}'", volume_name)
        else:
            logger.info("[kubectl patch deployment] Volume '{}' already present", volume_name)

        # Inject volumeMount on first container
        containers = spec.get("containers") or []
        if not containers:
            raise RuntimeError("No containers found in deployment spec")
        mount_entry = {"name": volume_name, "mountPath": mount_path}
        mounts = containers[0].get("volumeMounts") or []
        if not any(vm.get("name") == volume_name for vm in mounts):
            mounts.append(mount_entry)
            containers[0]["volumeMounts"] = mounts
            logger.info("[kubectl patch deployment] Added volumeMount '{}'", volume_name)
        else:
            logger.info("[kubectl patch deployment] VolumeMount '{}' already present", volume_name)

        # Strip fields that block kubectl apply
        metadata = doc.get("metadata", {})
        for field in ("resourceVersion", "uid", "creationTimestamp", "generation"):
            metadata.pop(field, None)
        annotations = metadata.get("annotations", {})
        annotations.pop("kubectl.kubernetes.io/last-applied-configuration", None)
        doc.pop("status", None)

        patched_yaml = _yaml.dump(doc, default_flow_style=False)

        # Write, apply, clean up
        write_cmd = f"cat << 'PATCH_EOF' > {remote_patch_file}\n{patched_yaml}\nPATCH_EOF"
        self._sesh.exec(write_cmd, block_name="kubectl write patch")
        apply_result = self._run(f"apply -f {remote_patch_file}", block_name="kubectl apply")
        self._sesh.exec(f"rm -f {remote_patch_file}", block_name="kubectl cleanup")

        if apply_result.exit_code != 0:
            raise RuntimeError(f"kubectl apply failed: {apply_result.stderr.strip()}")

        # Wait for rollout
        return self.rollout_status(
            resource=f"deployment/{deployment}", namespace=namespace, timeout=rollout_timeout
        )

    def patch_file_from_pod(
        self,
        *,
        deployment: str,
        source_file: str,
        dest_dir: str,
        replacements: dict[str, str],
        chmod: str = "777",
        namespace: str | None = None,
    ) -> str:
        """Copy a file from a pod to the VM host, apply text replacements, and set permissions.

        On re-runs where a hostPath mount makes the pod's source file and the
        host file the same inode, restores from the ``.bak`` backup on the host
        instead of ``kubectl exec ... cat`` (which can produce empty output
        during pod restarts).

        Args:
            deployment: Deployment name (used as ``deploy/{deployment}`` target).
            source_file: Absolute path to the source file inside the pod.
            dest_dir: Destination directory on the VM host.
            replacements: Dict mapping original text to replacement text.
            chmod: File permissions to set (default "777").
            namespace: Override the default namespace.

        Returns:
            The absolute path to the patched file on the host.

        Example::

            kube.patch_file_from_pod(
                deployment="web",
                source_file="/opt/app/handler.py",
                dest_dir="/root/patches",
                replacements={"get_host(req)": "'localhost:4000'"},
            )
        """
        ns = self._ns(namespace)
        source_basename = source_file.rsplit("/", 1)[-1]
        target_file = f"{dest_dir}/{source_basename}"
        backup = f"{target_file}.bak"

        # Create destination directory
        self._sesh.exec(f"mkdir -p {dest_dir}", block_name="kubectl patch file")

        # Back up existing file
        has_existing = (
            self._sesh.exec(f"test -s {target_file}", block_name="kubectl patch file").exit_code
            == 0
        )
        if has_existing:
            logger.info("[kubectl patch file] Backing up {} → {}", target_file, backup)
            self._sesh.exec(f"cp {target_file} {backup}", block_name="kubectl patch file")

        # Copy source: use backup if available (re-run safe), otherwise kubectl exec cat
        has_backup = (
            self._sesh.exec(f"test -s {backup}", block_name="kubectl patch file").exit_code == 0
        )

        if has_backup:
            logger.info("[kubectl patch file] Re-run detected — restoring from {}", backup)
            self._sesh.exec(f"cp {backup} {target_file}", block_name="kubectl patch file")
        else:
            # First run: verify source exists in pod, then copy
            verify = self._run(
                f"exec -n {ns} deploy/{deployment} -- test -f {source_file}",
                block_name="kubectl patch file",
            )
            if verify.exit_code != 0:
                raise RuntimeError(f"Source file not found in pod: {source_file}")

            temp_file = f"{target_file}.tmp"
            logger.info("[kubectl patch file] Copying {} from pod → {}", source_file, target_file)
            copy_cmd = (
                f"bash -c 'kubectl exec -n {ns} deploy/{deployment} "
                f"-- cat {source_file} > {temp_file}'"
            )
            self._sesh.exec(copy_cmd, block_name="kubectl patch file")
            self._sesh.exec(f"mv {temp_file} {target_file}", block_name="kubectl patch file")

        # Sanity check
        check = self._sesh.exec(
            f"test -s {target_file} && head -c 4 {target_file}", block_name="kubectl patch file"
        )
        if check.exit_code != 0 or not check.stdout.strip():
            raise RuntimeError(f"Copied file is missing or empty: {target_file}")

        # Apply text replacements via sed
        for original, patched in replacements.items():
            logger.info("[kubectl patch file] sed: {}... → {}...", original[:50], patched[:50])
            escaped_orig = original.replace("'", r"'\''")
            escaped_patched = patched.replace("'", r"'\''")
            self._sesh.exec(
                f"sed -i 's|{escaped_orig}|{escaped_patched}|g' {target_file}",
                block_name="kubectl patch file",
            )

        # Set permissions
        logger.info("[kubectl patch file] chmod {} {}", chmod, target_file)
        self._sesh.exec(f"chmod {chmod} {target_file}", block_name="kubectl patch file")

        logger.success("[kubectl patch file] Patched and ready: {}", target_file)
        return target_file


def connect(sesh: SSHSession, *, namespace: str) -> KubectlSession:
    """Create a bound kubectl session over an existing SSH connection.

    Args:
        sesh: An active SSHSession (from ``ssh.connect()``).
        namespace: Default Kubernetes namespace for all commands.

    Returns:
        A KubectlSession bound to the SSH connection and namespace.

    Example::

        with ssh.connect(ctx, container="my-app", host="10.0.0.1",
                         username="root", password=pw) as sesh:
            kube = kubectl.connect(sesh, namespace="prod")
            kube.get("deployment", name="web")
    """
    logger.info("kubectl session bound to namespace '{}'", namespace)
    return KubectlSession(sesh, namespace)
