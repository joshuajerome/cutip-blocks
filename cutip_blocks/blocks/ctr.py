"""ctr blocks — containerd CLI operations over SSH.

Note: ``ctr`` is the containerd CLI, not ``crictl`` (CRI CLI).
"""

from __future__ import annotations

from cutip_blocks.blocks.ssh import ExecResult, SSHSession
from cutip_blocks.decorator import block


@block(name="Image Import", category="ctr", action="image_import")
def image_import(
    ctx,
    sesh: SSHSession,
    *,
    tar: str,
    socket: str = "/run/k3s/containerd/containerd.sock",
    namespace: str = "k8s.io",
) -> ExecResult:
    """Run ``ctr -a <socket> -n=<namespace> image import <tar>``.

    Imports a container image tarball directly into containerd.

    Args:
        ctx: CutipContext.
        sesh: Active SSHSession.
        tar: Path to the .tar image on the remote host.
        socket: Containerd socket path.
        namespace: Containerd namespace.

    Example::

        ctr.image_import(ctx, sesh, tar="/tmp/my-image.tar")
    """
    result = sesh.exec(
        f"ctr -a {socket} -n={namespace} image import {tar}",
        block_name="ctr image import",
    )
    if result.exit_code != 0:
        raise RuntimeError(f"ctr image import failed: {result.stderr.strip()}")
    return result
