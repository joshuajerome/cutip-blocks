"""rsty — Rust-backed workflow blocks for cutip.

All Rust-implemented classes are re-exported here so consumers can write
``from rsty import SSHSession`` (public) instead of
``from rsty._core import SSHSession`` (touching internals).
"""

from rsty._core import (
    AuthError,
    CommandFailed,
    ConnectionError,
    ContainerExecResult,
    ContainerRuntime,
    CutipBlocksError,
    ExecResult,
    HttpResponse,
    KubectlSession,
    SSHSession,
    ShellSession,
    TimeoutError,
    ValidationError,
    base64_decode,
    base64_encode,
    container_connect,
    kubectl_connect,
    sign_rsa_sha256,
    ssh_connect,
)

__all__ = [
    # Classes (Rust-backed)
    "AuthError",
    "CommandFailed",
    "ConnectionError",
    "ContainerExecResult",
    "ContainerRuntime",
    "CutipBlocksError",
    "ExecResult",
    "HttpResponse",
    "KubectlSession",
    "SSHSession",
    "ShellSession",
    "TimeoutError",
    "ValidationError",
    # Factory functions
    "container_connect",
    "kubectl_connect",
    "ssh_connect",
    # Crypto helpers
    "base64_decode",
    "base64_encode",
    "sign_rsa_sha256",
]
