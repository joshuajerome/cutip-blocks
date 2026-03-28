"""Service blocks — readiness polling and container exit waiting."""

from __future__ import annotations

import time
from collections.abc import Callable
from typing import Any

from loguru import logger

from cutip_blocks.decorator import block


@block(name="Poll Until Ready", category="service", action="poll_until_ready")
def poll_until_ready(
    ctx,
    *,
    check_fn: Callable[[], bool],
    timeout: int = 30,
    interval: int = 1,
    description: str = "service",
) -> bool:
    """Poll ``check_fn()`` until it returns truthy or timeout is reached.

    Example::

        service.poll_until_ready(ctx,
            check_fn=lambda: ctx.container("redis").reload() is None
                and ctx.container("redis").status == "running",
            timeout=10, description="Redis")
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
def wait_for_exit(ctx, *, container_obj: Any, timeout: int = 60, interval: float = 0.5) -> int:
    """Poll ``container_obj.reload()`` + ``.status`` until container exits.

    Example::

        c = ctx.container("my-task"); c.start()
        service.wait_for_exit(ctx, container_obj=c, timeout=60)
    """
    logger.info("[Wait for Exit] Waiting for container to exit (timeout={}s)", timeout)
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        container_obj.reload()
        if container_obj.status in ("exited", "stopped"):
            logger.info("[Wait for Exit] Container exited: {}", container_obj.status)
            return 0
        time.sleep(interval)
    logger.warning("[Wait for Exit] Container did not exit within {}s", timeout)
    return -1
