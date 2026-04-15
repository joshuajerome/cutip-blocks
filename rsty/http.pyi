"""Type stubs for rsty.http — HTTP requests via reqwest."""

from typing import Any

class HttpResponse:
    """Result of an HTTP request."""

    status_code: int
    text: str
    ok: bool

    def json(self) -> dict[str, Any] | list[Any]:
        """Parse the response body as JSON."""
        ...

def get(
    url: str,
    *,
    headers: dict[str, str] | None = None,
    verify_tls: bool = True,
    timeout_s: int = 30,
) -> HttpResponse:
    """Send an HTTP GET request.

    Args:
        url: Request URL.
        headers: Optional headers.
        verify_tls: Verify TLS certificates (default True).
        timeout_s: Request timeout in seconds.
    """
    ...

def post(
    url: str,
    *,
    json: dict[str, Any] | None = None,
    body: str | None = None,
    headers: dict[str, str] | None = None,
    verify_tls: bool = True,
    timeout_s: int = 30,
) -> HttpResponse:
    """Send an HTTP POST request.

    Args:
        url: Request URL.
        json: JSON body (dict).
        body: Raw string body.
        headers: Optional headers.
        verify_tls: Verify TLS certificates.
        timeout_s: Request timeout in seconds.
    """
    ...

def put(
    url: str,
    *,
    json: dict[str, Any] | None = None,
    body: str | None = None,
    headers: dict[str, str] | None = None,
    verify_tls: bool = True,
    timeout_s: int = 30,
) -> HttpResponse:
    """Send an HTTP PUT request."""
    ...

def delete(
    url: str,
    *,
    headers: dict[str, str] | None = None,
    verify_tls: bool = True,
    timeout_s: int = 30,
) -> HttpResponse:
    """Send an HTTP DELETE request."""
    ...
