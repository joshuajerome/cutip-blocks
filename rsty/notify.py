"""Notify blocks — OS-native toast notifications via Rust."""

from __future__ import annotations

from rsty._core import send as _send

__all__ = ["send"]


def send(title: str, message: str, *, sound: bool = True) -> None:
    """Show an OS-native toast notification."""
    return _send(title, message, sound=sound)
