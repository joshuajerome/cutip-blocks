"""HTTP blocks — Python API wrapping Rust reqwest."""

from __future__ import annotations

from typing import Any

from rsty._core import HttpResponse
from rsty._core import delete as _delete
from rsty._core import get as _get
from rsty._core import post as _post
from rsty._core import put as _put

__all__ = ["get", "post", "put", "delete", "HttpResponse"]


def get(
    url: str,
    *,
    headers: dict[str, str] | None = None,
    verify_tls: bool = True,
    timeout_s: int = 30,
) -> HttpResponse:
    """Send an HTTP GET request."""
    return _get(url, headers=headers, verify_tls=verify_tls, timeout_s=timeout_s)


def post(
    url: str,
    *,
    json: dict[str, Any] | None = None,
    body: str | None = None,
    headers: dict[str, str] | None = None,
    verify_tls: bool = True,
    timeout_s: int = 30,
) -> HttpResponse:
    """Send an HTTP POST request with a JSON dict or raw body."""
    return _post(
        url,
        json=json,
        body=body,
        headers=headers,
        verify_tls=verify_tls,
        timeout_s=timeout_s,
    )


def put(
    url: str,
    *,
    json: dict[str, Any] | None = None,
    body: str | None = None,
    headers: dict[str, str] | None = None,
    verify_tls: bool = True,
    timeout_s: int = 30,
) -> HttpResponse:
    """Send an HTTP PUT request with a JSON dict or raw body."""
    return _put(
        url,
        json=json,
        body=body,
        headers=headers,
        verify_tls=verify_tls,
        timeout_s=timeout_s,
    )


def delete(
    url: str,
    *,
    headers: dict[str, str] | None = None,
    verify_tls: bool = True,
    timeout_s: int = 30,
) -> HttpResponse:
    """Send an HTTP DELETE request."""
    return _delete(url, headers=headers, verify_tls=verify_tls, timeout_s=timeout_s)
