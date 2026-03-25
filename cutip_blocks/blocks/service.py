"""Service blocks — daemon management and health polling."""

from __future__ import annotations

import time

from loguru import logger

from cutip_blocks.decorator import block


@block(name="Poll Until Ready", category="service", action="poll_until_ready")
def poll_until_ready(
    ctx,
    *,
    check_fn,
    timeout: int = 30,
    interval: int = 1,
    description: str = "service",
) -> bool:
    """Poll a callable until it returns truthy or timeout is reached.

    Args:
        check_fn: Callable that returns truthy when ready.
        timeout: Max wait time in seconds.
        interval: Seconds between polls.
        description: Label for log messages.
    """
    logger.info("[Poll Until Ready] Waiting for {} (timeout={}s)", description, timeout)
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            if check_fn():
                logger.info("[Poll Until Ready] {} is ready", description)
                return True
        except Exception:
            pass
        time.sleep(interval)

    logger.warning("[Poll Until Ready] {} timed out after {}s", description, timeout)
    return False


@block(name="Wait for Exit", category="service", action="wait_for_exit")
def wait_for_exit(
    ctx,
    *,
    container_obj,
    timeout: int = 60,
    interval: float = 0.5,
) -> int:
    """Wait for a container to exit and return exit code.

    Args:
        container_obj: Docker/Podman container object with .reload() and .status.
        timeout: Max wait time in seconds.
        interval: Seconds between polls.
    """
    logger.info("[Wait for Exit] Waiting for container to exit (timeout={}s)", timeout)
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        container_obj.reload()
        if container_obj.status in ("exited", "stopped"):
            logger.info("[Wait for Exit] Container exited with status: {}", container_obj.status)
            return 0
        time.sleep(interval)

    logger.warning("[Wait for Exit] Container did not exit within {}s", timeout)
    return -1
