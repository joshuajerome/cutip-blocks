"""Crypto blocks — RSA signing and base64 encoding via Rust."""

from __future__ import annotations

from rsty._core import base64_decode as _base64_decode
from rsty._core import base64_encode as _base64_encode
from rsty._core import sign_rsa_sha256 as _sign_rsa_sha256

__all__ = ["sign_rsa_sha256", "base64_encode", "base64_decode"]


def sign_rsa_sha256(key_path: str, data: str) -> str:
    """Sign ``data`` with an RSA PKCS#8 PEM key using SHA-256; return base64."""
    return _sign_rsa_sha256(key_path, data)


def base64_encode(data: str) -> str:
    """Encode a string as standard base64."""
    return _base64_encode(data)


def base64_decode(data: str) -> str:
    """Decode a standard base64 string back to UTF-8."""
    return _base64_decode(data)
