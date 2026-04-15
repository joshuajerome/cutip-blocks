"""Package management blocks — install/remove system packages."""

from cutip_blocks._core import install, remove, update

# Rename to avoid shadowing builtins in the module namespace
install = install
remove = remove
update = update

__all__ = ["install", "remove", "update"]
