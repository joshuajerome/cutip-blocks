"""Kubernetes blocks — kubectl operations over an SSH session."""

from __future__ import annotations

import json as _json
import tempfile

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
    output: str = "jsonpath='{.items[0].metadata.name}'",
) -> ExecResult:
    """Get pod(s) for a deployment by label selector."""
    result = _kubectl(
        sesh,
        f"get pod -n {namespace} -l app={deployment} -o {output}",
        block_name="Get Pod",
    )
    if result.exit_code != 0:
        raise RuntimeError(
            f"No pods found for deployment {deployment} in {namespace}: {result.stderr.strip()}"
        )
    return result


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
    # Get pod name
    pod_result = get_pod(
        ctx, sesh, namespace=namespace, deployment=deployment,
        output="jsonpath='{.items[0].metadata.name}'",
    )
    pod_name = pod_result.stdout.strip().strip("'")

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

    sesh.exec(f"cat > {remote_file} << 'CUTIP_EOF'\n{patched_yaml}CUTIP_EOF", block_name="Patch Deployment")
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
