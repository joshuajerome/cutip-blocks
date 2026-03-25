"""Tests for file blocks."""

from cutip_blocks.blocks.file import is_empty


def test_is_empty_none():
    assert is_empty(None) is True


def test_is_empty_empty_string():
    assert is_empty("") is True


def test_is_empty_whitespace():
    assert is_empty("   \n  ") is True


def test_is_empty_content():
    assert is_empty("hello") is False


def test_is_empty_bytes():
    assert is_empty(b"") is True
    assert is_empty(b"data") is False
