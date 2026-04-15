"""Tests for Rust-backed kubectl module."""

from rsty._core import KubectlSession, kubectl_connect, SSHSession


def test_imports():
    """Verify Rust types are importable."""
    assert KubectlSession is not None
    assert kubectl_connect is not None


def test_kubectl_wrapper_importable():
    """Python wrapper module should import cleanly."""
    from rsty import kubectl
    assert hasattr(kubectl, "connect")


def test_kubectl_session_methods():
    """KubectlSession should have all expected methods."""
    expected = [
        "get", "get_secret_value", "find_pod", "exec",
        "cat_file", "cp", "apply", "rollout_status",
        "patch_deployment", "patch_file_from_pod",
    ]
    for method in expected:
        assert hasattr(KubectlSession, method), f"Missing method: {method}"
