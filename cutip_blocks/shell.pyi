"""Type stubs for cutip_blocks.shell — local shell command execution."""

from cutip_blocks._core import ExecResult

def run(
    cmd: str,
    *,
    cwd: str | None = None,
    env: dict[str, str] | None = None,
    check: bool = True,
    stream: bool = False,
) -> ExecResult:
    """Execute a shell command on the local host.

    Args:
        cmd: Command string to execute.
        cwd: Working directory.
        env: Additional environment variables (merged with current env).
        check: If True (default), raise CommandFailed on non-zero exit.
        stream: If True, output streams to console in real-time. ExecResult stdout/stderr will be empty.

    Returns:
        ExecResult with exit_code, stdout, stderr.

    Raises:
        CommandFailed: If check=True and exit code is non-zero.
        ConnectionError: If the command cannot be spawned.
    """
    ...
