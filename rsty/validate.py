"""Validate blocks — environment and path checks."""

from __future__ import annotations

from rsty._core import env_var_set as _env_var_set
from rsty._core import ip_valid as _ip_valid
from rsty._core import path_exists as _path_exists

__all__ = ["path_exists", "env_var_set", "ip_valid"]


def path_exists(path: str) -> bool:
    """Return True if a file or directory exists at ``path``."""
    return _path_exists(path)


def env_var_set(name: str) -> bool:
    """Return True if the environment variable ``name`` is set and non-empty."""
    return _env_var_set(name)


def ip_valid(addr: str) -> bool:
    """Return True if ``addr`` parses as a valid IPv4 or IPv6 address."""
    return _ip_valid(addr)
