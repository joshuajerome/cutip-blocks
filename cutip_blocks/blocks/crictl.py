"""crictl/ctr blocks — container runtime interface operations over SSH."""

from __future__ import annotations

from cutip_blocks.blocks.ssh import ExecResult, SSHSession
from cutip_blocks.decorator import block


@block(name="Image List", category="crictl", action="image_ls")
def image_ls(ctx, sesh: SSHSession) -> ExecResult:
    """List container images via crictl."""
    return sesh.exec("crictl image ls", block_name="Image List")


@block(name="Image Remove", category="crictl", action="image_rm")
def image_rm(ctx, sesh: SSHSession, *, image: str) -> ExecResult:
    """Remove a container image via crictl."""
    result = sesh.exec(f"crictl image rm {image}", block_name="Image Remove")
    if result.exit_code != 0:
        raise RuntimeError(f"crictl image rm failed: {result.stderr.strip()}")
    return result


@block(name="Image Import", category="crictl", action="image_import")
def image_import(
    ctx,
    sesh: SSHSession,
    *,
    tar: str,
    socket: str = "/run/k3s/containerd/containerd.sock",
    namespace: str = "k8s.io",
) -> ExecResult:
    """Import a container image tarball via ctr.

    Args:
        tar: Path to the .tar image on the remote host.
        socket: Containerd socket path.
        namespace: Containerd namespace.
    """
    result = sesh.exec(
        f"ctr -a {socket} -n={namespace} image import {tar}",
        block_name="Image Import",
    )
    if result.exit_code != 0:
        raise RuntimeError(f"ctr image import failed: {result.stderr.strip()}")
    return result
