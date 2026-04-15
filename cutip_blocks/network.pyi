"""Type stubs for cutip_blocks.network — Docker/Podman network management."""

def create(name: str, *, driver: str = "bridge", subnet: str | None = None, gateway: str | None = None) -> None:
    """Create a Docker/Podman network."""
    ...

def remove(name: str) -> None:
    """Remove a Docker/Podman network."""
    ...

def exists(name: str) -> bool:
    """Check if a Docker/Podman network exists."""
    ...
