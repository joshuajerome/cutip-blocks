"""Kubernetes blocks — kubectl operations over an SSH session."""

from __future__ import annotations

import json as _json

from loguru import logger

from cutip_blocks.blocks.ssh import ExecResult, SSHSession
from cutip_blocks.decorator import block


def _kubectl(sesh: SSHSession, cmd: str, *, block_name: str) -> ExecResult:
    """Run a kubectl command over the SSH session."""
    full_cmd = f"kubectl {cmd}"
    return sesh.exec(full_cmd, block_name=block_name)


@block(name="Get Deployment", category="k8s", action="get_deployment")
def get_deployment(
    ctx,
    sesh: SSHSession,
    *,
    namespace: str,
    deployment: str,
    output: str = "name",
) -> ExecResult:
    """Verify a deployment exists. Raises on failure."""
    result = _kubectl(
        sesh,
        f"get deployment -n {namespace} {deployment} -o {output}",
        block_name="Get Deployment",
    )
    if result.exit_code != 0:
        raise RuntimeError(
            f"Deployment {deployment} not found in namespace {namespace}: {result.stderr.strip()}"
        )
    return result


@block(name="Get Secret", category="k8s", action="get_secret")
def get_secret(
    ctx,
    sesh: SSHSession,
    *,
    namespace: str,
    secret: str,
    output: str = "name",
) -> ExecResult:
    """Get a Kubernetes secret. Use output='jsonpath=...' for specific fields.

    Example — decode a password::

        result = k8s.get_secret(ctx, sesh, namespace="ns", secret="my-creds",
                                output='jsonpath="{.data.password}" | base64 -d')
    """
    result = _kubectl(
        sesh,
        f"get secret -n {namespace} {secret} -o {output}",
        block_name="Get Secret",
    )
    if result.exit_code != 0:
        raise RuntimeError(
            f"Secret {secret} not found in namespace {namespace}: {result.stderr.strip()}"
        )
    return result


@block(name="Get Pod", category="k8s", action="get_pod")
def get_pod(
    ctx,
    sesh: SSHSession,
    *,
    namespace: str,
    deployment: str,
    output: str | None = None,
) -> ExecResult:
    """Find a pod matching a deployment name via JSON inspection.

    Queries all pods in the namespace as JSON, finds pods whose name
    starts with the deployment name (excluding the replica-set hash suffix).

    - 1 match → uses it
    - Multiple matches → selects the first Running pod, warns about others
    - 0 matches → errors with list of available pods for debugging

    If ``output`` is specified, runs a follow-up query with ``-o <output>``
    against the discovered pod name.
    """

    # Get all pods as JSON (no grep dependency)
    result = _kubectl(
        sesh,
        f"get pod -n {namespace} -o json",
        block_name="Get Pod",
    )
    if result.exit_code != 0:
        raise RuntimeError(f"Failed to list pods in namespace {namespace}: {result.stderr.strip()}")

    try:
        data = _json.loads(result.stdout)
    except _json.JSONDecodeError:
        raise RuntimeError(f"Failed to parse pod list JSON from namespace {namespace}")

    items = data.get("items", [])

    # Find pods whose name starts with the deployment name
    matches: list[dict] = []
    all_pod_names: list[str] = []
    for pod in items:
        pod_name = pod.get("metadata", {}).get("name", "")
        all_pod_names.append(pod_name)
        if pod_name.startswith(deployment):
            phase = pod.get("status", {}).get("phase", "Unknown")
            matches.append({"name": pod_name, "phase": phase})

    if not matches:
        available = ", ".join(all_pod_names[:10]) or "(none)"
        raise RuntimeError(
            f"No pods matching '{deployment}' in namespace {namespace}. Available pods: {available}"
        )

    # Prefer Running pods
    running = [m for m in matches if m["phase"] == "Running"]
    selected = running[0] if running else matches[0]

    if len(matches) > 1:
        logger.warning(
            "[Get Pod] Multiple pods match '{}': {}. Using: {} ({})",
            deployment,
            ", ".join(f"{m['name']} ({m['phase']})" for m in matches),
            selected["name"],
            selected["phase"],
        )
    else:
        logger.info("[Get Pod] Found pod: {} ({})", selected["name"], selected["phase"])

    # Build result with the pod name in stdout
    pod_result = ExecResult(
        exit_code=0,
        stdout=selected["name"],
        stderr="",
    )

    if output:
        pod_result = _kubectl(
            sesh,
            f"get pod -n {namespace} {selected['name']} -o {output}",
            block_name="Get Pod",
        )
        if pod_result.exit_code != 0:
            raise RuntimeError(
                f"Failed to query pod {selected['name']}: {pod_result.stderr.strip()}"
            )

    return pod_result


@block(name="Exec in Pod", category="k8s", action="exec")
def exec(
    ctx,
    sesh: SSHSession,
    *,
    namespace: str,
    deployment: str,
    cmd: str,
) -> ExecResult:
    """Execute a command inside a pod via kubectl exec.

    Routes through ``deploy/`` so kubectl picks the pod automatically.
    Supports pipes: ``cmd="cat /app/config.py | head -5"``
    """
    result = _kubectl(
        sesh,
        f"exec -n {namespace} deploy/{deployment} -- bash -c '{cmd}'",
        block_name="Exec in Pod",
    )
    return result


@block(name="Copy from Pod", category="k8s", action="cp")
def cp(
    ctx,
    sesh: SSHSession,
    *,
    namespace: str,
    deployment: str,
    src: str,
    dest: str,
) -> ExecResult:
    """Copy a file from a pod to the VM filesystem.

    Resolves the pod name from the deployment label, then runs kubectl cp.
    """
    # Get pod name via grep
    pod_result = get_pod(ctx, sesh, namespace=namespace, deployment=deployment)
    pod_name = pod_result.stdout.strip()

    result = _kubectl(
        sesh,
        f"cp {namespace}/{pod_name}:{src} {dest}",
        block_name="Copy from Pod",
    )
    if result.exit_code != 0:
        raise RuntimeError(f"kubectl cp failed: {result.stderr.strip()}")
    return result


@block(name="Apply", category="k8s", action="apply")
def apply(
    ctx,
    sesh: SSHSession,
    *,
    file: str,
) -> ExecResult:
    """Apply a YAML manifest via kubectl apply -f."""
    result = _kubectl(sesh, f"apply -f {file}", block_name="Apply")
    if result.exit_code != 0:
        raise RuntimeError(f"kubectl apply failed: {result.stderr.strip()}")
    return result


@block(name="Patch Deployment", category="k8s", action="patch_deployment")
def patch_deployment(
    ctx,
    sesh: SSHSession,
    *,
    namespace: str,
    deployment: str,
    volume_name: str,
    host_path: str,
    mount_path: str,
) -> ExecResult:
    """Patch a deployment to add a hostPath volume mount.

    1. Fetches current deployment YAML
    2. Adds/updates the volume + volumeMount
    3. Applies the patched manifest
    4. Waits for rollout
    """
    # Get current deployment
    result = _kubectl(
        sesh,
        f"get deployment -n {namespace} {deployment} -o yaml",
        block_name="Patch Deployment",
    )
    if result.exit_code != 0:
        raise RuntimeError(f"Cannot fetch deployment: {result.stderr.strip()}")

    import yaml

    deploy_yaml = yaml.safe_load(result.stdout)
    spec = deploy_yaml["spec"]["template"]["spec"]

    # Ensure volumes list exists
    if "volumes" not in spec:
        spec["volumes"] = []

    # Add or update volume
    volumes = spec["volumes"]
    existing_vol = next((v for v in volumes if v.get("name") == volume_name), None)
    if existing_vol:
        existing_vol["hostPath"] = {"path": host_path}
    else:
        volumes.append({"name": volume_name, "hostPath": {"path": host_path}})

    # Add or update volumeMount on first container
    containers = spec["containers"]
    if containers:
        c = containers[0]
        if "volumeMounts" not in c:
            c["volumeMounts"] = []
        mounts = c["volumeMounts"]
        existing_mount = next((m for m in mounts if m.get("name") == volume_name), None)
        if existing_mount:
            existing_mount["mountPath"] = mount_path
        else:
            mounts.append({"name": volume_name, "mountPath": mount_path})

    # Write patched YAML to remote temp file and apply
    patched_yaml = yaml.dump(deploy_yaml, default_flow_style=False)
    remote_file = f"/tmp/cutip-patch-{deployment}.yaml"

    sesh.exec(
        f"cat > {remote_file} << 'CUTIP_EOF'\n{patched_yaml}CUTIP_EOF",
        block_name="Patch Deployment",
    )
    apply_result = _kubectl(sesh, f"apply -f {remote_file}", block_name="Patch Deployment")
    sesh.exec(f"rm -f {remote_file}", block_name="Patch Deployment")

    if apply_result.exit_code != 0:
        raise RuntimeError(f"kubectl apply failed: {apply_result.stderr.strip()}")

    return apply_result


@block(name="Rollout Status", category="k8s", action="rollout_status")
def rollout_status(
    ctx,
    sesh: SSHSession,
    *,
    namespace: str,
    deployment: str,
    timeout: int = 120,
) -> ExecResult:
    """Wait for a deployment rollout to complete."""
    result = _kubectl(
        sesh,
        f"rollout status deployment -n {namespace} {deployment} --timeout={timeout}s",
        block_name="Rollout Status",
    )
    if result.exit_code != 0:
        raise RuntimeError(
            f"Rollout failed for {deployment} in {namespace}: {result.stderr.strip()}"
        )
    logger.info("[Rollout Status] {} rolled out successfully", deployment)
    return result
