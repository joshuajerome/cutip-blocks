"""File blocks — Python API wrapping Rust file operations."""

from __future__ import annotations

from typing import Any

from rsty._core import copy as _copy
from rsty._core import copy_tree as _copy_tree
from rsty._core import is_empty as _is_empty
from rsty._core import mkdir as _mkdir
from rsty._core import read_json as _read_json
from rsty._core import read_yaml as _read_yaml
from rsty._core import replace as _replace
from rsty._core import write_json as _write_json
from rsty._core import write_yaml as _write_yaml

__all__ = [
    "copy",
    "copy_tree",
    "read_yaml",
    "write_yaml",
    "read_json",
    "write_json",
    "replace",
    "mkdir",
    "is_empty",
]


def copy(src: str, dest: str) -> str:
    """Copy a single file, creating parent directories as needed."""
    return _copy(src, dest)


def copy_tree(src: str, dest: str, clean: bool = False) -> str:
    """Copy a directory recursively; if ``clean`` is True, remove ``dest`` first."""
    return _copy_tree(src, dest, clean)


def read_yaml(path: str) -> dict[str, Any]:
    """Read and parse a YAML file into a Python dict."""
    return _read_yaml(path)


def write_yaml(path: str, data: dict[str, Any]) -> str:
    """Serialize a dict to YAML and write it to ``path``."""
    return _write_yaml(path, data)


def read_json(path: str) -> dict[str, Any] | list[Any]:
    """Read and parse a JSON file into Python data."""
    return _read_json(path)


def write_json(path: str, data: dict[str, Any]) -> str:
    """Serialize a dict to pretty JSON and write it to ``path``."""
    return _write_json(path, data)


def replace(path: str, old: str, new: str) -> str:
    """Replace all occurrences of ``old`` with ``new`` in the file at ``path``."""
    return _replace(path, old, new)


def mkdir(path: str) -> str:
    """Create a directory (and parents) if it doesn't exist."""
    return _mkdir(path)


def is_empty(value: str | None) -> bool:
    """Return True if ``value`` is None or whitespace-only."""
    return _is_empty(value)
