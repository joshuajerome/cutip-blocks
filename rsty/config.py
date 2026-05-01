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
    globals: dict[str, str] | None = None,
) -> str:
    """Replace ``{{ vars.key }}``, ``{{ paths.key }}``, ``{{ secrets.key }}``,
    and ``{{ globals.key.nested }}`` placeholders in text.

    Both spaced (``{{ ns.key }}``) and unspaced (``{{ns.key}}``) forms are
    supported. ``paths``, ``secrets``, and ``globals`` are optional.

    ``globals`` keys are pre-flattened dotted paths (e.g. ``passwords.v22``).
    Use ``ctx.globals`` from a cutip workflow to get the flattened form for
    free.
    """
    return _substitute_vars(text, vars, secrets, paths, globals)
