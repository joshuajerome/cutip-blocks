"""Type stubs for rsty.config — template rendering."""

def render_template(template: str, vars: dict[str, str]) -> str:
    """Replace {{ key }} placeholders in a template string.

    Args:
        template: Template string with {{ key }} placeholders.
        vars: Key-value pairs for substitution.

    Returns:
        Rendered string.
    """
    ...

def substitute_vars(
    text: str,
    vars: dict[str, str],
    secrets: dict[str, str] | None = None,
) -> str:
    """Replace {{ vars.key }} and {{ secrets.key }} placeholders.

    Args:
        text: Text with placeholders.
        vars: Variables dict.
        secrets: Optional secrets dict.

    Returns:
        Text with all placeholders resolved.
    """
    ...
