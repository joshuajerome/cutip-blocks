"""Network blocks — Python API wrapping Rust network operations."""

from __future__ import annotations

from rsty._core import ContainerRuntime
from rsty._core import create as _create
from rsty._core import exists as _exists
from rsty._core import remove as _remove

__all__ = ["create", "remove", "exists"]


def create(
    runtime: ContainerRuntime,
    name: str,
    *,
    driver: str = "bridge",
    subnet: str | None = None,
    gateway: str | None = None,
) -> None:
    """Create a Docker/Podman network."""
    return _create(runtime, name, driver=driver, subnet=subnet, gateway=gateway)


def remove(runtime: ContainerRuntime, name: str) -> None:
    """Remove a Docker/Podman network by name."""
    return _remove(runtime, name)


def exists(runtime: ContainerRuntime, name: str) -> bool:
    """Return True if a network with the given name exists."""
    return _exists(runtime, name)
