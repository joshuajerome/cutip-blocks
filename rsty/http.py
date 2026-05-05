"""HTTP blocks — Python API wrapping Rust reqwest."""

from __future__ import annotations

from typing import Any

from rsty._core import HttpResponse, HttpSession
from rsty._core import delete as _delete
from rsty._core import get as _get
from rsty._core import http_session as _http_session
from rsty._core import post as _post
from rsty._core import put as _put

__all__ = ["get", "post", "put", "delete", "session", "HttpResponse", "HttpSession"]


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
    form: dict[str, str] | None = None,
    headers: dict[str, str] | None = None,
    verify_tls: bool = True,
    timeout_s: int = 30,
) -> HttpResponse:
    """Send an HTTP POST request.

    Body source is one of (mutually exclusive):
      * ``json`` — dict serialized as ``application/json``.
      * ``form`` — dict serialized as ``application/x-www-form-urlencoded``.
      * ``body`` — raw string body (caller sets Content-Type via ``headers``).
    """
    return _post(
        url,
        json=json,
        body=body,
        form=form,
        headers=headers,
        verify_tls=verify_tls,
        timeout_s=timeout_s,
    )


def put(
    url: str,
    *,
    json: dict[str, Any] | None = None,
    body: str | None = None,
    form: dict[str, str] | None = None,
    headers: dict[str, str] | None = None,
    verify_tls: bool = True,
    timeout_s: int = 30,
) -> HttpResponse:
    """Send an HTTP PUT request. Same body semantics as :func:`post`."""
    return _put(
        url,
        json=json,
        body=body,
        form=form,
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


def session(
    *,
    verify_tls: bool = True,
    follow_redirects: bool = True,
    timeout_s: int = 30,
    headers: dict[str, str] | None = None,
) -> HttpSession:
    """Open a persistent HTTP session.

    The session keeps a cookie jar across requests and lets you choose
    whether to auto-follow redirects. Useful for OAuth2 / Keycloak flows
    where you need to read the ``Location`` header on a 302 yourself.

    Use as a context manager::

        with http.session(verify_tls=False, follow_redirects=False) as sesh:
            page = sesh.get(login_url)
            sesh.post(action_url, form={"username": u, "password": p})
    """
    return _http_session(
        verify_tls=verify_tls,
        follow_redirects=follow_redirects,
        timeout_s=timeout_s,
        headers=headers,
    )
