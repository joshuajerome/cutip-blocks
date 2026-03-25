"""Validate blocks — precondition checks."""

from __future__ import annotations

import ipaddress
import os
from pathlib import Path

from loguru import logger

from cutip_blocks.decorator import block


@block(name="Path Exists", category="validate", action="path_exists")
def path_exists(ctx, *, path: str | Path, description: str = "") -> bool:
    """Check that a filesystem path exists."""
    path = Path(path)
    label = description or str(path)
    exists = path.exists()
    if exists:
        logger.info("[Path Exists] {} ✓", label)
    else:
        logger.warning("[Path Exists] {} — not found", label)
    return exists


@block(name="Env Var Set", category="validate", action="env_var_set")
def env_var_set(ctx, *, key: str) -> str:
    """Check that an environment variable is set and non-empty. Returns value."""
    value = os.environ.get(key, "")
    if value:
        logger.info("[Env Var Set] {} ✓", key)
    else:
        logger.warning("[Env Var Set] {} — not set or empty", key)
    return value


@block(name="IP Valid", category="validate", action="ip_valid")
def ip_valid(ctx, *, address: str) -> bool:
    """Validate an IP address format."""
    try:
        ipaddress.ip_address(address.strip())
        logger.info("[IP Valid] {} ✓", address)
        return True
    except ValueError:
        logger.warning("[IP Valid] {} — invalid format", address)
        return False
