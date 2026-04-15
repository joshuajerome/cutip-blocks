"""Structured error types for cutip-blocks.

Error hierarchy:
    CutipBlocksError (base)
    ├── ConnectionError   — transient connection failure (retryable)
    ├── TimeoutError      — operation exceeded time limit (retryable)
    ├── AuthError         — authentication failed (not retryable)
    ├── CommandFailed     — non-zero exit code (triggers on_fail)
    └── ValidationError   — bad input (not retryable)
"""

from rsty._core import (
    CutipBlocksError,
    ConnectionError,
    TimeoutError,
    AuthError,
    CommandFailed,
    ValidationError,
)

__all__ = [
    "CutipBlocksError",
    "ConnectionError",
    "TimeoutError",
    "AuthError",
    "CommandFailed",
    "ValidationError",
]
