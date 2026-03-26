"""Config blocks — template rendering and variable substitution."""

from __future__ import annotations

from pathlib import Path

from loguru import logger

from cutip_blocks.decorator import block


@block(name="Render Template", category="config", action="render_template")
def render_template(
    ctx,
    *,
    template: str | Path,
    dest: str | Path,
    variables: dict[str, str],
) -> Path:
    """Render a template file with {{ var }} substitution.

    Args:
        template: Path to template file.
        dest: Path to write rendered output.
        variables: Dict of variable name → value.
    """
    template, dest = Path(template), Path(dest)
    logger.info("[Render Template] {} → {} ({} vars)", template, dest, len(variables))

    text = template.read_text(encoding="utf-8")
    for key, value in variables.items():
        text = text.replace(f"{{{{ {key} }}}}", str(value))
        text = text.replace(f"{{{{{key}}}}}", str(value))

    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(text, encoding="utf-8")
    return dest


@block(name="Substitute Vars", category="config", action="substitute_vars")
def substitute_vars(ctx, *, text: str, variables: dict[str, str]) -> str:
    """Replace {{ var }} placeholders in a string.

    Args:
        text: Input string with placeholders.
        variables: Dict of variable name → value.
    """
    result = text
    for key, value in variables.items():
        result = result.replace(f"{{{{ {key} }}}}", str(value))
        result = result.replace(f"{{{{{key}}}}}", str(value))
    return result
