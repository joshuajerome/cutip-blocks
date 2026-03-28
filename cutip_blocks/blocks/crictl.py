"""crictl blocks — CRI operations over SSH. For containerd (ctr), see ctr.py."""

from __future__ import annotations

import warnings

from cutip_blocks.blocks.ssh import ExecResult, SSHSession
from cutip_blocks.decorator import block


@block(name="Image List", category="crictl", action="image_ls")
def image_ls(ctx, sesh: SSHSession) -> ExecResult:
    """Run ``crictl image ls``."""
    return sesh.exec("crictl image ls", block_name="crictl image ls")


@block(name="Image Remove", category="crictl", action="image_rm")
def image_rm(ctx, sesh: SSHSession, *, image: str) -> ExecResult:
    """Run ``crictl image rm <image>``."""
    result = sesh.exec(f"crictl image rm {image}", block_name="crictl image rm")
    if result.exit_code != 0:
        raise RuntimeError(f"crictl image rm failed: {result.stderr.strip()}")
    return result


@block(name="Image Import (deprecated)", category="crictl", action="image_import")
def image_import(ctx, sesh: SSHSession, *, tar: str, **kwargs) -> ExecResult:
    """.. deprecated:: 0.2.0 Use ``ctr.image_import()`` — this runs ctr, not crictl."""
    warnings.warn(
        "crictl.image_import() is deprecated. Use ctr.image_import().",
        DeprecationWarning,
        stacklevel=2,
    )
    from cutip_blocks.blocks.ctr import image_import as _ctr

    return _ctr(ctx, sesh, tar=tar, **kwargs)
