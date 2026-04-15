"""SSH blocks — Python API wrapping Rust SSH implementation."""

from __future__ import annotations

from contextlib import contextmanager
from typing import Generator

from rsty._core import ExecResult, SSHSession, ssh_connect


@contextmanager
def connect(
    *,
    host: str,
    username: str,
    password: str,
    port: int = 22,
) -> Generator[SSHSession, None, None]:
    """Open a persistent SSH session.

    Args:
        host: Remote host IP or hostname.
        username: SSH username.
        password: SSH password.
        port: SSH port (default 22).

    Yields:
        SSHSession with .exec(cmd) and .probe() methods.

    Example::

        with ssh.connect(host="10.0.0.1", username="root", password=pw) as sesh:
            sesh.probe()
            result = sesh.exec("ls /tmp")
            print(result.stdout)
    """
    sesh = ssh_connect(host=host, username=username, password=password, port=port)
    try:
        yield sesh
    finally:
        sesh.close()
