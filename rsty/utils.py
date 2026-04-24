"""Utility functions."""

from __future__ import annotations

from rsty._core import is_empty as _is_empty

__all__ = ["is_empty"]


def is_empty(value: str | None) -> bool:
    """Return True if ``value`` is None or whitespace-only."""
    return _is_empty(value)
