"""Type stubs for rsty.validate — basic validation checks."""

def path_exists(path: str) -> bool:
    """Check if a file or directory exists."""
    ...

def env_var_set(name: str) -> bool:
    """Check if an environment variable is set and non-empty."""
    ...

def ip_valid(addr: str) -> bool:
    """Check if a string is a valid IPv4 or IPv6 address."""
    ...
