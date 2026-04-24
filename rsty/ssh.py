"""SSH blocks — Python API wrapping Rust SSH implementation."""

from __future__ import annotations

from contextlib import contextmanager
from typing import Generator

from rsty._core import ExecResult, SSHSession, ShellSession, ssh_connect


def open(
    *,
    host: str,
    username: str,
    password: str,
    port: int = 22,
) -> SSHSession:
    """Open a persistent SSH session.

    Caller is responsible for calling .close() when done.

    Args:
        host: Remote host IP or hostname.
        username: SSH username.
        password: SSH password.
        port: SSH port (default 22).

    Returns:
        SSHSession with .exec(cmd), .probe(), .shell(), .close() methods.

    Example::

        sesh = ssh.open(host="10.0.0.1", username="root", password=pw)
        sesh.probe()
        sesh.close()
    """
    return ssh_connect(host=host, username=username, password=password, port=port)


@contextmanager
def connect(
    *,
    host: str,
    username: str,
    password: str,
    port: int = 22,
) -> Generator[SSHSession, None, None]:
    """Open a persistent SSH session (context manager).

    Automatically closes on exit.

    Example::

        with ssh.connect(host="10.0.0.1", username="root", password=pw) as sesh:
            sesh.probe()
            result = sesh.exec("ls /tmp")
    """
    sesh = ssh_connect(host=host, username=username, password=password, port=port)
    try:
        yield sesh
    finally:
        sesh.close()
