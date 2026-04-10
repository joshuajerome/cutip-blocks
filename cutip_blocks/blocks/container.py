"""Backward compat — re-exports from cutip_blocks.container"""
from cutip_blocks.container import *
from cutip_blocks.container import connect
from cutip_blocks._core import ContainerRuntime, ContainerExecResult

# Legacy API: container.start(ctx, container="name")
# The old blocks took ctx as first arg. Provide a compat shim.
def start(ctx, *, container: str):
    """Start a container by name (legacy API)."""
    c = ctx.container(container)
    c.start()
    return c

def stop(ctx, *, container: str):
    """Stop a container by name (legacy API)."""
    ctx.container(container).stop()

def remove(ctx, *, container: str):
    """Remove a container by name (legacy API)."""
    ctx.container(container).remove()
