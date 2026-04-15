"""Type stubs for rsty.file — filesystem operations."""

from typing import Any

def copy(src: str, dest: str) -> str:
    """Copy a single file. Creates parent directories if needed.

    Args:
        src: Source file path.
        dest: Destination file path.

    Returns:
        The destination path.
    """
    ...

def copy_tree(src: str, dest: str, clean: bool = False) -> str:
    """Copy a directory recursively.

    Args:
        src: Source directory path.
        dest: Destination directory path.
        clean: If True, remove dest before copying.

    Returns:
        The destination path.
    """
    ...

def read_yaml(path: str) -> dict[str, Any]:
    """Read and parse a YAML file.

    Args:
        path: Path to the YAML file.

    Returns:
        Parsed YAML as a Python dict.
    """
    ...

def write_yaml(path: str, data: dict[str, Any]) -> str:
    """Write a Python dict to a YAML file.

    Args:
        path: Output file path.
        data: Data to serialize.

    Returns:
        The file path.
    """
    ...

def read_json(path: str) -> dict[str, Any] | list[Any]:
    """Read and parse a JSON file.

    Args:
        path: Path to the JSON file.

    Returns:
        Parsed JSON as a Python dict or list.
    """
    ...

def write_json(path: str, data: dict[str, Any]) -> str:
    """Write a Python dict to a JSON file.

    Args:
        path: Output file path.
        data: Data to serialize.

    Returns:
        The file path.
    """
    ...

def replace(path: str, old: str, new: str) -> str:
    """Replace all occurrences of a string in a file.

    Args:
        path: File to modify.
        old: String to find.
        new: Replacement string.

    Returns:
        The file path.
    """
    ...

def mkdir(path: str) -> str:
    """Create a directory and all parent directories.

    Args:
        path: Directory path to create.

    Returns:
        The directory path.
    """
    ...

def is_empty(value: str | None) -> bool:
    """Check if a value is empty, None, or whitespace-only.

    Args:
        value: String to check.

    Returns:
        True if empty/None/whitespace.
    """
    ...
