"""@block decorator and BlockMeta dataclass."""

from __future__ import annotations

from dataclasses import dataclass
from functools import wraps
from typing import Callable

_BLOCK_ATTR = "_cutip_block_meta"


@dataclass(frozen=True)
class BlockMeta:
    """Metadata attached to a @block-decorated function."""

    name: str
    category: str
    action: str


def block(name: str, category: str, action: str) -> Callable:
    """Decorator that marks a function as a CUTIP block.

    Usage::

        @block(name="Get Secret", category="k8s", action="get_secret")
        def get_secret(ctx, sesh, *, namespace, secret, output="name"):
            ...
    """
    meta = BlockMeta(name=name, category=category, action=action)

    def decorator(fn: Callable) -> Callable:
        @wraps(fn)
        def wrapper(*args, **kwargs):
            return fn(*args, **kwargs)

        setattr(wrapper, _BLOCK_ATTR, meta)
        return wrapper

    return decorator
