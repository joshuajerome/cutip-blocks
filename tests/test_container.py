"""Tests for Rust-backed container runtime."""

import pytest
from rsty import container


def test_imports():
    assert container.connect is not None
    assert container.ContainerRuntime is not None
    assert container.ContainerExecResult is not None


def test_connect():
    """Connect to Docker daemon (requires Docker running)."""
    try:
        rt = container.connect()
        assert rt is not None
    except RuntimeError as e:
        pytest.skip(f"Docker not available: {e}")


def test_exists_nonexistent():
    """Check a container that doesn't exist."""
    try:
        rt = container.connect()
        assert rt.exists("cutip-test-nonexistent-xyz") is False
    except RuntimeError:
        pytest.skip("Docker not available")
