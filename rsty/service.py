"""Service blocks — readiness polling and container wait."""

from __future__ import annotations

from rsty._core import ContainerRuntime
from rsty._core import poll_until_ready as _poll_until_ready
from rsty._core import wait_for_exit as _wait_for_exit

__all__ = ["poll_until_ready", "wait_for_exit"]


def poll_until_ready(
    url: str,
    *,
    retries: int = 30,
    interval_s: int = 1,
    verify_tls: bool = True,
) -> None:
    """Poll an HTTP endpoint until it returns a success status code."""
    return _poll_until_ready(
        url, retries=retries, interval_s=interval_s, verify_tls=verify_tls
    )


def wait_for_exit(
    runtime: ContainerRuntime,
    container_name: str,
    *,
    timeout_s: int = 120,
) -> int:
    """Wait for a container to exit and return its status code."""
    return _wait_for_exit(runtime, container_name, timeout_s=timeout_s)
