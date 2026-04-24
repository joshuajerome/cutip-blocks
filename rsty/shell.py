"""Shell blocks — execute commands on the local host via Rust."""

from __future__ import annotations

from rsty._core import ExecResult
from rsty._core import run as _run

__all__ = ["run"]


def run(
    cmd: str,
    *,
    cwd: str | None = None,
    env: dict[str, str] | None = None,
    check: bool = True,
    stream: bool = False,
) -> ExecResult:
    """Run a shell command on the local host.

    If ``stream`` is True, stdout/stderr are inherited and the ExecResult
    buffers will be empty.
    """
    return _run(cmd, cwd=cwd, env=env, check=check, stream=stream)
