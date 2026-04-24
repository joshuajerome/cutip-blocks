"""Service management blocks — systemd service control."""

from __future__ import annotations

from rsty._core import ExecResult
from rsty._core import disable as _disable
from rsty._core import enable as _enable
from rsty._core import is_active as _is_active
from rsty._core import restart as _restart
from rsty._core import start as _start
from rsty._core import status as _status
from rsty._core import stop as _stop

__all__ = [
    "start",
    "stop",
    "restart",
    "enable",
    "disable",
    "is_active",
    "status",
]


def start(service: str) -> None:
    """Start a systemd service."""
    return _start(service)


def stop(service: str) -> None:
    """Stop a systemd service."""
    return _stop(service)


def restart(service: str) -> None:
    """Restart a systemd service."""
    return _restart(service)


def enable(service: str) -> None:
    """Enable a systemd service (start on boot)."""
    return _enable(service)


def disable(service: str) -> None:
    """Disable a systemd service."""
    return _disable(service)


def is_active(service: str) -> bool:
    """Return True if the systemd service is currently active."""
    return _is_active(service)


def status(service: str) -> ExecResult:
    """Return the raw ``systemctl status`` output for a service."""
    return _status(service)
