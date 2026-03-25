"""File blocks — filesystem operations."""

from __future__ import annotations

import json
import shutil
from pathlib import Path

import yaml
from loguru import logger

from cutip_blocks.decorator import block


@block(name="Copy File", category="file", action="copy")
def copy(ctx, *, src: str | Path, dest: str | Path) -> Path:
    """Copy a single file."""
    src, dest = Path(src), Path(dest)
    logger.info("[Copy File] {} → {}", src, dest)
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy(src, dest)
    return dest


@block(name="Copy Tree", category="file", action="copy_tree")
def copy_tree(ctx, *, src: str | Path, dest: str | Path, clean: bool = False) -> Path:
    """Copy a directory recursively."""
    src, dest = Path(src), Path(dest)
    logger.info("[Copy Tree] {} → {}{}", src, dest, " (clean)" if clean else "")
    if clean and dest.exists():
        shutil.rmtree(dest)
    shutil.copytree(src, dest, dirs_exist_ok=not clean)
    return dest


@block(name="Read YAML", category="file", action="read_yaml")
def read_yaml(ctx, *, path: str | Path) -> dict:
    """Read and parse a YAML file."""
    path = Path(path)
    logger.info("[Read YAML] {}", path)
    with path.open(encoding="utf-8") as fh:
        return yaml.safe_load(fh) or {}


@block(name="Write YAML", category="file", action="write_yaml")
def write_yaml(ctx, *, path: str | Path, data: dict) -> Path:
    """Write a dict to a YAML file."""
    path = Path(path)
    logger.info("[Write YAML] {}", path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(yaml.dump(data, default_flow_style=False, sort_keys=False), encoding="utf-8")
    return path


@block(name="Read JSON", category="file", action="read_json")
def read_json(ctx, *, path: str | Path) -> dict:
    """Read and parse a JSON file."""
    path = Path(path)
    logger.info("[Read JSON] {}", path)
    with path.open(encoding="utf-8") as fh:
        return json.load(fh)


@block(name="Write JSON", category="file", action="write_json")
def write_json(ctx, *, path: str | Path, data: dict) -> Path:
    """Write a dict to a JSON file."""
    path = Path(path)
    logger.info("[Write JSON] {}", path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2), encoding="utf-8")
    return path


@block(name="Replace in File", category="file", action="replace")
def replace(ctx, *, path: str | Path, old: str, new: str) -> Path:
    """Replace a string in a file (all occurrences)."""
    path = Path(path)
    logger.info("[Replace in File] {} :: '{}' → '{}'", path, old[:60], new[:60])
    text = path.read_text(encoding="utf-8")
    if old not in text:
        logger.warning("[Replace in File] Pattern not found in {}", path)
    updated = text.replace(old, new)
    path.write_text(updated, encoding="utf-8")
    return path


@block(name="Is Empty", category="file", action="is_empty")
def is_empty(value: str | bytes | None) -> bool:
    """Check if a value (file content, command output) is empty or None."""
    if value is None:
        return True
    if isinstance(value, bytes):
        return len(value.strip()) == 0
    return len(value.strip()) == 0
