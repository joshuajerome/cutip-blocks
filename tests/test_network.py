"""Tests for network blocks."""

from cutip_blocks import network


def test_imports():
    assert network.create is not None
    assert network.remove is not None
    assert network.exists is not None
