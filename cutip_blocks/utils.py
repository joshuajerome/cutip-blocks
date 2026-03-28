"""Pure utility functions — not blocks, no @block decorator."""

from __future__ import annotations


def is_empty(value: str | bytes | None) -> bool:
    """Check if a value (file content, command output) is empty or None.

    Args:
        value: String, bytes, or None to check.

    Returns:
        True if value is None, empty, or whitespace-only.

    Example::

        from cutip_blocks.utils import is_empty
        if is_empty(result.stdout):
            raise RuntimeError("No output")
    """
    if value is None:
        return True
    if isinstance(value, bytes):
        return len(value.strip()) == 0
    return len(value.strip()) == 0
