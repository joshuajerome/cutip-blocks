"""Container blocks — Python API wrapping Rust container runtime."""

from cutip_blocks._core import ContainerRuntime, ContainerExecResult, container_connect


def connect(socket: str | None = None) -> ContainerRuntime:
    """Connect to Docker/Podman daemon.

    Args:
        socket: Optional path to Docker socket. Defaults to auto-detect.

    Returns:
        ContainerRuntime with start/stop/remove/exec/pull methods.
    """
    return container_connect(socket=socket)
