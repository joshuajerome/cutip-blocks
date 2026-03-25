"""Container blocks — lifecycle and exec operations."""

from __future__ import annotations

import sys

from loguru import logger

from cutip_blocks.decorator import block


@block(name="Start Container", category="container", action="start")
def start(ctx, *, container: str):
    """Start a container by name."""
    logger.info("[Start Container] {}", container)
    c = ctx.container(container)
    c.start()
    return c


@block(name="Stop Container", category="container", action="stop")
def stop(ctx, *, container: str):
    """Stop a running container."""
    logger.info("[Stop Container] {}", container)
    ctx.container(container).stop()


@block(name="Remove Container", category="container", action="remove")
def remove(ctx, *, container: str):
    """Remove a container."""
    logger.info("[Remove Container] {}", container)
    ctx.container(container).remove()


@block(name="Exec in Container", category="container", action="exec")
def exec(ctx, *, container: str, cmd: str | list[str], detach: bool = False):
    """Execute a command inside a container.

    Args:
        container: Container name.
        cmd: Command string or list.
        detach: If True, fire and forget.
    """
    display = cmd if isinstance(cmd, str) else " ".join(cmd)
    logger.info("[Exec in Container] {} :: {}", container, display)
    c = ctx.container(container)
    exit_code, output = c.exec_run(cmd, detach=detach)
    if not detach and exit_code and exit_code != 0:
        logger.warning("[Exec in Container] {} :: {} [exit {}]", container, display, exit_code)
    return exit_code, output


@block(name="Exec Stream", category="container", action="exec_stream")
def exec_stream(ctx, *, container: str, cmd: str | list[str]):
    """Execute a command inside a container with streaming output.

    Streams stdout to sys.stdout in real time.
    """
    display = cmd if isinstance(cmd, str) else " ".join(cmd)
    logger.info("[Exec Stream] {} :: {}", container, display)
    c = ctx.container(container)
    exit_code, output = c.exec_run(cmd, stream=True, demux=False)
    for chunk in output:
        sys.stdout.write(chunk.decode(errors="replace"))
        sys.stdout.flush()
    if exit_code and exit_code != 0:
        logger.warning("[Exec Stream] {} :: {} [exit {}]", container, display, exit_code)
    return exit_code
