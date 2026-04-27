"""SSH blocks — Python API wrapping Rust SSH implementation."""

from __future__ import annotations

import re
import time
from contextlib import contextmanager
from dataclasses import dataclass
from typing import Callable, Generator

from rsty._core import ExecResult, SSHSession, ShellSession, ssh_connect


# Sentinels we inject into the remote shell to detect command completion + PID.
# Random-ish strings to avoid collision with normal output.
_EXIT_SENTINEL = "__rsty_exit_8f2e1d__"
_PID_SENTINEL = "__rsty_pid_8f2e1d__"
_EXIT_RE = re.compile(rf"{re.escape(_EXIT_SENTINEL)}=(\d+)")
_PID_RE = re.compile(rf"{re.escape(_PID_SENTINEL)}=(\d+)")


@dataclass
class StreamResult:
    """Result of an exec_stream call."""
    exit_code: int
    lines: list[str]


def open(
    *,
    host: str,
    username: str,
    password: str,
    port: int = 22,
) -> SSHSession:
    """Open a persistent SSH session.

    Caller is responsible for calling .close() when done.
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
    """
    sesh = ssh_connect(host=host, username=username, password=password, port=port)
    try:
        yield sesh
    finally:
        sesh.close()


def exec_stream(
    sesh: SSHSession,
    cmd: str,
    *,
    on_line: Callable[[str], None] | None = None,
    timeout: int = 3600,
    line_timeout: int = 120,
) -> StreamResult:
    """Run a command in a shell session, streaming stdout line-by-line.

    Each line read from the remote shell is passed to ``on_line`` (if given)
    and accumulated in the returned ``StreamResult.lines``. The command is
    wrapped with an exit-code sentinel so we know when it finishes.

    Args:
        sesh: An open SSHSession.
        cmd: Shell command to run on the remote host.
        on_line: Callback invoked once per line of output (no trailing newline).
        timeout: Total wall-clock timeout in seconds (default 1 hour).
        line_timeout: Timeout for any single line read (default 2 min).
            Raise this if the command goes silent for long stretches.

    Returns:
        StreamResult with exit_code and the captured lines (sentinel + echo
        excluded).

    Raises:
        TimeoutError: if the command does not complete within ``timeout``.
        RuntimeError: if the shell session closes before the sentinel arrives.

    Example::

        sesh = ssh.open(host=h, username=u, password=p)
        result = ssh.exec_stream(
            sesh,
            "make blueprint-manager",
            on_line=lambda line: print(f"[build] {line}"),
        )
        if result.exit_code != 0:
            raise RuntimeError("Build failed")
    """
    shell = sesh.shell()
    deadline = time.monotonic() + timeout
    lines: list[str] = []

    try:
        # Wrap so we get exit code on its own line. The {} group's exit code
        # propagates to $? for the printf.
        wrapped = f"{{ {cmd}; }}; printf '\\n{_EXIT_SENTINEL}=%s\\n' \"$?\""
        shell.send_line(wrapped)

        while True:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise TimeoutError(f"exec_stream timed out after {timeout}s")

            line = shell.read_line(timeout=min(line_timeout, int(remaining) + 1))
            line = line.rstrip("\r\n")

            # Skip the echo of the wrapped command itself — its sentinel text
            # contains "=%s" (the printf format), not "=<digit>".
            if _EXIT_SENTINEL in line and not _EXIT_RE.search(line):
                continue

            m = _EXIT_RE.search(line)
            if m:
                pre = line[: m.start()].rstrip()
                if pre:
                    lines.append(pre)
                    if on_line is not None:
                        on_line(pre)
                return StreamResult(exit_code=int(m.group(1)), lines=lines)

            lines.append(line)
            if on_line is not None:
                on_line(line)
    finally:
        try:
            shell.close()
        except Exception:
            pass


def shell_with_pid(
    sesh: SSHSession,
    cmd: str,
    *,
    line_timeout: int = 30,
) -> tuple[int, ShellSession]:
    """Run cmd in a shell session, return (remote_pid, shell).

    The command is launched in the background and its PID echoed back.
    Caller is responsible for calling ``shell.close()`` when done.

    The returned shell can be used to:
      - Read more output (the foreground shell stays connected)
      - Send signals via ``sesh.exec(f"kill -TERM {pid}")`` from a separate session

    Args:
        sesh: An open SSHSession.
        cmd: Shell command to run.
        line_timeout: Timeout for reading the PID announcement line.

    Returns:
        (pid, shell) — the remote PID of the launched process, and the shell.

    Example::

        pid, shell = ssh.shell_with_pid(sesh, "long_running_thing")
        # Later, from another SSH session:
        ctrl = ssh.open(host=h, username=u, password=p)
        ctrl.exec(f"kill -TERM {pid}")
        shell.close()
    """
    shell = sesh.shell()
    try:
        wrapped = (
            f"({cmd}) & "
            f"printf '\\n{_PID_SENTINEL}=%s\\n' \"$!\"; "
            f"wait $!; "
            f"printf '\\n{_EXIT_SENTINEL}=%s\\n' \"$?\""
        )
        shell.send_line(wrapped)

        # Find the PID line; tolerate up to a few echoed/intermediate lines.
        for _ in range(20):
            line = shell.read_line(timeout=line_timeout)
            m = _PID_RE.search(line)
            if m:
                return int(m.group(1)), shell
        raise RuntimeError("PID sentinel not received from remote shell")
    except Exception:
        try:
            shell.close()
        except Exception:
            pass
        raise


def upload(sesh: SSHSession, local: str, remote: str) -> None:
    """Upload local file to remote path via SFTP.

    Streams in chunks (handles arbitrarily large files). Overwrites the
    remote file if it exists.

    Example::

        ssh.upload(sesh, "/tmp/build.tar.gz", "/tmp/build.tar.gz")
    """
    sesh.upload(local, remote)


def download(sesh: SSHSession, remote: str, local: str) -> None:
    """Download remote file to local path via SFTP.

    Streams in chunks (handles arbitrarily large files). Overwrites the
    local file if it exists.

    Example::

        ssh.download(sesh, "/var/log/app.log", "/tmp/app.log")
    """
    sesh.download(remote, local)


def signal(sesh: SSHSession, pid: int, sig: str = "TERM") -> None:
    """Send a signal to a remote PID via ``kill``.

    Args:
        sesh: An open SSHSession (any session to the same host as the PID).
        pid: Remote process ID.
        sig: Signal name without the SIG prefix (default ``TERM``).

    Example::

        ssh.signal(sesh, pid)            # SIGTERM
        ssh.signal(sesh, pid, "KILL")    # SIGKILL
    """
    sesh.exec(f"kill -{sig} {pid}")
