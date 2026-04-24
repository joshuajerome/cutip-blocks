"""Package management blocks — install/remove system packages."""

from __future__ import annotations

from rsty._core import install as _install
from rsty._core import remove as _remove
from rsty._core import update as _update

__all__ = ["install", "remove", "update"]


def install(packages: list[str]) -> None:
    """Install system packages. Auto-detects apt/dnf/yum/apk."""
    return _install(packages)


def remove(packages: list[str]) -> None:
    """Remove system packages. Auto-detects apt/dnf/yum/apk."""
    return _remove(packages)


def update() -> None:
    """Update the system package index. Auto-detects apt/dnf/yum/apk."""
    return _update()
