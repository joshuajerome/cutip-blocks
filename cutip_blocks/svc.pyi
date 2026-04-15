"""Type stubs for cutip_blocks.svc — systemd service management."""

from cutip_blocks._core import ExecResult

def start(service: str) -> None:
    """Start a systemd service."""
    ...

def stop(service: str) -> None:
    """Stop a systemd service."""
    ...

def restart(service: str) -> None:
    """Restart a systemd service."""
    ...

def enable(service: str) -> None:
    """Enable a systemd service (start on boot)."""
    ...

def disable(service: str) -> None:
    """Disable a systemd service."""
    ...

def is_active(service: str) -> bool:
    """Check if a systemd service is active (running)."""
    ...

def status(service: str) -> ExecResult:
    """Get the status of a systemd service.

    Returns:
        ExecResult with exit_code, stdout (status output), stderr.
    """
    ...
