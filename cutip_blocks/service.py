"""Service blocks — readiness polling and container wait."""

from cutip_blocks._core import poll_until_ready, wait_for_exit

__all__ = ["poll_until_ready", "wait_for_exit"]
