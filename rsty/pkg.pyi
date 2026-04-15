"""Type stubs for rsty.pkg — package management."""

def install(packages: list[str]) -> None:
    """Install system packages. Auto-detects apt/dnf/yum/apk.

    Args:
        packages: List of package names to install.

    Raises:
        CommandFailed: If install fails.
        ValidationError: If no supported package manager found.
    """
    ...

def remove(packages: list[str]) -> None:
    """Remove system packages. Auto-detects apt/dnf/yum/apk.

    Args:
        packages: List of package names to remove.
    """
    ...

def update() -> None:
    """Update the package index. Auto-detects apt/dnf/yum/apk."""
    ...
