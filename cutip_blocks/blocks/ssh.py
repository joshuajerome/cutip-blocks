"""SSH blocks — session management, exec, and probe.

Usage::

    with ssh.connect(ctx, container="my-app", host="10.0.0.1",
                     username="root", password=pw) as sesh:
        sesh.probe()
        sesh.exec("ls /tmp")
"""

from __future__ import annotations

import warnings
from contextlib import contextmanager
from dataclasses import dataclass

from loguru import logger

from cutip_blocks.decorator import block


@dataclass
class ExecResult:
    """Result of a remote command execution."""

    exit_code: int
    stdout: str
    stderr: str = ""


class SSHSession:
    """Persistent SSH session. Created via ``ssh.connect()``.

    Methods:
        exec(cmd): Run ``ssh <user>@<host> '<cmd>'``
        probe(): Run ``ssh <user>@<host> 'whoami'``
        close(): Close the connection
    """

    def __init__(
        self,
        host: str,
        username: str,
        password: str,
        *,
        container_runtime=None,
        container_name: str | None = None,
        port: int = 22,
    ) -> None:
        self._host = host
        self._username = username
        self._port = port
        self._container_runtime = container_runtime
        self._container_name = container_name
        self._redact: set[str] = {password}
        self._client = None

    def _connect(self) -> None:
        import paramiko

        self._client = paramiko.SSHClient()
        self._client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
        self._client.connect(
            hostname=self._host,
            port=self._port,
            username=self._username,
            password=next(iter(self._redact)),
            timeout=30,
            allow_agent=False,
            look_for_keys=False,
        )

    def close(self) -> None:
        """Close the SSH connection."""
        if self._client:
            try:
                self._client.close()
            except Exception:
                pass
            self._client = None

    def exec(self, cmd: str, *, block_name: str = "") -> ExecResult:
        """Run ``ssh <user>@<host> '<cmd>'``.

        Args:
            cmd: Shell command string. Pipes, redirects, and chaining are valid.
            block_name: Optional prefix for log messages.

        Returns:
            ExecResult with exit_code, stdout, and stderr.

        Example::

            sesh.exec("whoami")
            sesh.exec("kubectl get pods -n prod | grep web")
        """
        if self._client is None:
            raise RuntimeError("SSH session is not connected")

        prefix = f"[{block_name}] " if block_name else ""
        display_cmd = self._redact_str(cmd)
        logger.info("{}ssh {}@{} :: {}", prefix, self._username, self._host, display_cmd)

        _, stdout_ch, stderr_ch = self._client.exec_command(cmd, timeout=300)
        stdout = stdout_ch.read().decode("utf-8", errors="replace")
        stderr = stderr_ch.read().decode("utf-8", errors="replace")
        exit_code = stdout_ch.channel.recv_exit_status()

        if exit_code != 0:
            logger.warning(
                "{}ssh {}@{} :: {} [exit {}]",
                prefix,
                self._username,
                self._host,
                display_cmd,
                exit_code,
            )

        return ExecResult(exit_code=exit_code, stdout=stdout, stderr=stderr)

    def probe(self) -> ExecResult:
        """Run ``ssh <user>@<host> 'whoami'`` to verify connectivity.

        Returns:
            ExecResult with the username in stdout.

        Example::

            sesh.probe()  # logs: "Authenticated as: root"
        """
        result = self.exec("whoami", block_name="SSH Probe")
        if result.exit_code != 0:
            raise RuntimeError(f"SSH probe failed: {result.stderr.strip()}")
        logger.info("[SSH Probe] Authenticated as: {}", result.stdout.strip())
        return result

    def _redact_str(self, text: str) -> str:
        result = text
        for secret in self._redact:
            if secret and secret in result:
                result = result.replace(secret, "****")
        return result


@contextmanager
def connect(ctx, *, container: str, host: str, username: str, password: str, port: int = 22):
    """Open a persistent SSH session through a container.

    Args:
        ctx: CutipContext.
        container: Container name with network access to the host.
        host: Remote host IP or hostname.
        username: SSH username.
        password: SSH password (redacted from all logs).
        port: SSH port (default 22).

    Yields:
        SSHSession with ``.exec(cmd)`` and ``.probe()`` methods.

    Example::

        with ssh.connect(ctx, container="my-app", host="10.0.0.1",
                         username="root", password=pw) as sesh:
            sesh.probe()
            sesh.exec("ls /tmp")
    """
    sesh = SSHSession(
        host=host,
        username=username,
        password=password,
        port=port,
        container_runtime=getattr(ctx, "runtime", None),
        container_name=container,
    )
    logger.info("Opening SSH session to {}@{} via container '{}'", username, host, container)
    sesh._connect()
    try:
        yield sesh
    finally:
        sesh.close()
        logger.info("Closed SSH session to {}@{}", username, host)


# Backward compatibility
session = connect


@block(name="SSH Exec", category="ssh", action="exec")
def exec(ctx, sesh: SSHSession, *, cmd: str) -> ExecResult:
    """.. deprecated:: 0.2.0 Use ``sesh.exec(cmd)`` instead."""
    warnings.warn("ssh.exec() is deprecated. Use sesh.exec(cmd).", DeprecationWarning, stacklevel=2)
    return sesh.exec(cmd, block_name="SSH Exec")


@block(name="SSH Probe", category="ssh", action="probe")
def probe(ctx, sesh: SSHSession) -> ExecResult:
    """.. deprecated:: 0.2.0 Use ``sesh.probe()`` instead."""
    warnings.warn("ssh.probe() is deprecated. Use sesh.probe().", DeprecationWarning, stacklevel=2)
    return sesh.probe()
