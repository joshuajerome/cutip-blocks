"""Type stubs for rsty.template — template rendering."""

def render(src: str, vars: dict[str, str], dest: str | None = None) -> str:
    """Render a template file, replacing {{ key }} placeholders.

    Args:
        src: Path to the template file.
        vars: Key-value pairs for substitution.
        dest: If provided, write rendered output to this path.

    Returns:
        Rendered string.
    """
    ...

def render_string(text: str, vars: dict[str, str]) -> str:
    """Render a template string, replacing {{ key }} placeholders.

    Args:
        text: Template string.
        vars: Key-value pairs for substitution.

    Returns:
        Rendered string.
    """
    ...

def check(src: str, vars: dict[str, str]) -> list[str]:
    """Check for missing template variables.

    Args:
        src: Path to the template file.
        vars: Available variables.

    Returns:
        List of placeholder keys not found in vars.
    """
    ...
