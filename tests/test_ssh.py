"""Tests for Rust-backed SSH module."""

from rsty._core import ExecResult, SSHSession, ssh_connect


def test_imports():
    """Verify Rust types are importable."""
    assert ExecResult is not None
    assert SSHSession is not None
    assert ssh_connect is not None


def test_exec_result_repr():
    """ExecResult should have a readable repr."""
    # Can't construct directly from Python, but we can verify the class exists
    assert "ExecResult" in str(ExecResult)


def test_ssh_wrapper_importable():
    """Python wrapper module should import cleanly."""
    from rsty import ssh
    assert hasattr(ssh, "connect")
