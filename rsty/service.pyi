"""Type stubs for rsty.service — polling and readiness checks."""

def poll_until_ready(url: str, *, timeout: int = 60, interval: int = 2) -> bool:
    """Poll a URL until it returns a successful response.

    Args:
        url: URL to poll.
        timeout: Maximum wait time in seconds.
        interval: Seconds between attempts.

    Returns:
        True when ready.

    Raises:
        TimeoutError: If the URL doesn't become ready within timeout.
    """
    ...

def wait_for_exit(name: str, *, timeout: int = 300) -> int:
    """Wait for a container to exit.

    Args:
        name: Container name.
        timeout: Maximum wait time in seconds.

    Returns:
        Container exit code.
    """
    ...
