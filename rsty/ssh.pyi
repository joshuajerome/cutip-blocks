"""Type stubs for rsty.ssh — SSH session management."""

from contextlib import contextmanager
from typing import Generator

class ExecResult:
    """Result of a remote command execution."""

    exit_code: int
    stdout: str
    stderr: str

class SSHSession:
    """Persistent SSH session. Created via ssh.connect()."""

    def exec(self, cmd: str) -> ExecResult:
        """Execute a remote command.

        Args:
            cmd: Command string to execute on the remote host.

        Returns:
            ExecResult with exit_code, stdout, stderr.
        """
        ...

    def probe(self) -> ExecResult:
        """Run 'whoami' to verify SSH connectivity.

        Returns:
            ExecResult with the authenticated username.

        Raises:
            RuntimeError: If probe fails.
        """
        ...

    def close(self) -> None:
        """Close the SSH connection."""
        ...

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
    """
    ...
