"""Validate blocks — precondition checks."""

from __future__ import annotations

import ipaddress
import os
import warnings
from pathlib import Path

from loguru import logger

from cutip_blocks.decorator import block


@block(name="Path Exists", category="validate", action="path_exists")
def path_exists(ctx, *, path: str | Path, description: str = "") -> bool:
    """Check that a filesystem path exists via ``Path.exists()``.

    Example::

        validate.path_exists(ctx, path="/etc/config.yaml")
    """
    path = Path(path)
    label = description or str(path)
    exists = path.exists()
    if exists:
        logger.info("[Path Exists] {} ✓", label)
    else:
        logger.warning("[Path Exists] {} — not found", label)
    return exists


@block(name="Env Var Exists", category="validate", action="env_var_exists")
def env_var_exists(ctx, *, key: str) -> str:
    """Check that an env var exists and is non-empty via ``os.environ.get()``.

    Example::

        validate.env_var_exists(ctx, key="SSH_KEY_PATH")
    """
    value = os.environ.get(key, "")
    if value:
        logger.info("[Env Var Exists] {} ✓", key)
    else:
        logger.warning("[Env Var Exists] {} — not set or empty", key)
    return value


@block(name="IP Format", category="validate", action="ip_format")
def ip_format(ctx, *, address: str) -> bool:
    """Validate IP address format via ``ipaddress.ip_address()``. Checks format only.

    Example::

        validate.ip_format(ctx, address="10.0.0.1")
    """
    try:
        ipaddress.ip_address(address.strip())
        logger.info("[IP Format] {} ✓", address)
        return True
    except ValueError:
        logger.warning("[IP Format] {} — invalid format", address)
        return False


# Backward compatibility
def env_var_set(ctx, *, key: str) -> str:
    """.. deprecated:: 0.2.0 Use ``validate.env_var_exists()``."""
    warnings.warn("validate.env_var_set() → env_var_exists()", DeprecationWarning, stacklevel=2)
    return env_var_exists(ctx, key=key)


def ip_valid(ctx, *, address: str) -> bool:
    """.. deprecated:: 0.2.0 Use ``validate.ip_format()``."""
    warnings.warn("validate.ip_valid() → ip_format()", DeprecationWarning, stacklevel=2)
    return ip_format(ctx, address=address)
