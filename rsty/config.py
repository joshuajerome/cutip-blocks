"""Config blocks — template rendering and variable substitution."""

from __future__ import annotations

from rsty._core import render_template as _render_template
from rsty._core import substitute_vars as _substitute_vars

__all__ = ["render_template", "substitute_vars"]


def render_template(template: str, vars: dict[str, str]) -> str:
    """Replace ``{{ key }}`` placeholders in a template string."""
    return _render_template(template, vars)


def substitute_vars(
    text: str,
    vars: dict[str, str],
    secrets: dict[str, str] | None = None,
    paths: dict[str, str] | None = None,
) -> str:
    """Replace ``{{ vars.key }}``, ``{{ paths.key }}``, and ``{{ secrets.key }}``
    placeholders in text.

    Both spaced (``{{ ns.key }}``) and unspaced (``{{ns.key}}``) forms are
    supported. ``paths`` and ``secrets`` are optional — omit them to leave
    those placeholders untouched.
    """
    return _substitute_vars(text, vars, secrets, paths)
