"""BlockRegistry — discovers all @block-decorated functions."""

from __future__ import annotations

import importlib
import pkgutil
from typing import Callable

from cutip_blocks.decorator import BlockMeta, _BLOCK_ATTR


class BlockRegistry:
    """Registry of all discovered blocks.

    Usage::

        registry = BlockRegistry.discover()
        for meta, fn in registry.blocks:
            print(f"{meta.category}.{meta.action} — {meta.name}")
    """

    def __init__(self) -> None:
        self._blocks: list[tuple[BlockMeta, Callable]] = []

    @property
    def blocks(self) -> list[tuple[BlockMeta, Callable]]:
        return list(self._blocks)

    def get(self, category: str, action: str) -> tuple[BlockMeta, Callable] | None:
        """Look up a block by category and action."""
        for meta, fn in self._blocks:
            if meta.category == category and meta.action == action:
                return meta, fn
        return None

    def categories(self) -> list[str]:
        """Return sorted list of unique categories."""
        return sorted({meta.category for meta, _ in self._blocks})

    @classmethod
    def discover(cls) -> BlockRegistry:
        """Scan cutip_blocks.blocks.* for all @block-decorated functions."""
        registry = cls()

        import cutip_blocks.blocks as blocks_pkg

        for _importer, modname, _ispkg in pkgutil.walk_packages(
            blocks_pkg.__path__, prefix=blocks_pkg.__name__ + "."
        ):
            try:
                module = importlib.import_module(modname)
            except Exception:
                continue

            for attr_name in dir(module):
                obj = getattr(module, attr_name)
                if callable(obj):
                    meta = getattr(obj, _BLOCK_ATTR, None)
                    if isinstance(meta, BlockMeta):
                        registry._blocks.append((meta, obj))

        return registry
