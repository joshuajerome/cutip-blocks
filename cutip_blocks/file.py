"""File blocks — Python API wrapping Rust file operations."""

from cutip_blocks._core import (
    copy,
    copy_tree,
    read_yaml,
    write_yaml,
    read_json,
    write_json,
    replace,
    is_empty,
)

__all__ = [
    "copy", "copy_tree",
    "read_yaml", "write_yaml",
    "read_json", "write_json",
    "replace", "is_empty",
]
