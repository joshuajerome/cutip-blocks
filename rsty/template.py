"""Template rendering blocks — variable substitution in files and strings."""

from __future__ import annotations

from rsty._core import check as _check
from rsty._core import render as _render
from rsty._core import render_string as _render_string

__all__ = ["render", "render_string", "check"]


def render(
    src: str,
    vars: dict[str, str],
    dest: str | None = None,
) -> str:
    """Render a template file, replacing ``{{ key }}`` placeholders.

    If ``dest`` is given, writes the rendered output and returns the path;
    otherwise returns the rendered string.
    """
    return _render(src, vars, dest)


def render_string(text: str, vars: dict[str, str]) -> str:
    """Render a template string, replacing ``{{ key }}`` placeholders."""
    return _render_string(text, vars)


def check(src: str, vars: dict[str, str]) -> list[str]:
    """Return the list of ``{{ key }}`` placeholders in ``src`` missing from ``vars``."""
    return _check(src, vars)
