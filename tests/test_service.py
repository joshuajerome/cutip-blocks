"""Tests for service blocks."""

from cutip_blocks import service


def test_imports():
    assert service.poll_until_ready is not None
    assert service.wait_for_exit is not None
