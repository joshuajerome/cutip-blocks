"""JS module blocks — read / write simple module.exports object literal files.

Use cases: webpack proxy configs, Babel configs, Jest configs, ESLint
configs — anything where the JS file is essentially a JSON5 object
behind a `module.exports = ...;` (or `const X = ...; module.exports = X;`)
wrapper.

Boundaries:
  - Object literal must be JSON5 (single quotes, unquoted keys, trailing
    commas, comments are all OK). No function values, computed keys,
    env-var references, or imports — those would require a real JS
    engine.
  - The writer preserves the input wrapper style: if the file used
    `const NAME = ...; module.exports = NAME;`, the rewrite keeps the
    same keyword + identifier. Otherwise it emits `module.exports = ...;`.
"""

from __future__ import annotations

from pathlib import Path

from rsty._core import js_read_module as _js_read_module
from rsty._core import js_write_module as _js_write_module

__all__ = ["read_module", "write_module"]


def read_module(path: str | Path) -> dict:
    """Parse a JS module file and return the exported object as a dict.

    Raises RuntimeError if the file lacks `module.exports` or the body
    isn't valid JSON5.
    """
    return _js_read_module(str(path))


def write_module(path: str | Path, data: dict) -> str:
    """Write a dict back as a JS module.

    If ``path`` already exists, the wrapper style is preserved (named
    declaration vs direct ``module.exports``). For new files, defaults
    to the direct form.

    Returns the resolved path string.
    """
    return _js_write_module(str(path), data)
