"""Config blocks — template rendering."""

from __future__ import annotations

from pathlib import Path

from loguru import logger

from cutip_blocks.decorator import block


@block(name="Render Template", category="config", action="render_template")
def render_template(
    ctx, *, template: str | Path, dest: str | Path, variables: dict[str, str]
) -> Path:
    """Read a template file, replace ``{{ var }}`` placeholders, write to dest.

    Args:
        ctx: CutipContext.
        template: Path to template file with ``{{ var }}`` placeholders.
        dest: Path to write the rendered output.
        variables: Dict of variable name → replacement value.

    Example::

        config.render_template(ctx,
            template="dhcp/dhcpd.conf.tpl", dest="dhcp/dhcpd.conf",
            variables={"SUBNET": "10.89.0.0/16", "GATEWAY": "10.89.0.1"})
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
