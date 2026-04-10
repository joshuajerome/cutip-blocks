"""Tests for Rust-backed file operations."""

import json
import os
import tempfile

from cutip_blocks import file


def test_json_roundtrip():
    with tempfile.TemporaryDirectory() as td:
        data = {"key": "value", "num": 42, "nested": {"a": [1, 2, 3]}}
        file.write_json(f"{td}/test.json", data)
        loaded = file.read_json(f"{td}/test.json")
        assert loaded["key"] == "value"
        assert loaded["num"] == 42
        assert loaded["nested"]["a"] == [1, 2, 3]


def test_yaml_roundtrip():
    with tempfile.TemporaryDirectory() as td:
        data = {"name": "test", "items": ["a", "b"]}
        file.write_yaml(f"{td}/test.yaml", data)
        loaded = file.read_yaml(f"{td}/test.yaml")
        assert loaded["name"] == "test"
        assert loaded["items"] == ["a", "b"]


def test_copy_file():
    with tempfile.TemporaryDirectory() as td:
        open(f"{td}/src.txt", "w").write("hello")
        file.copy(f"{td}/src.txt", f"{td}/dst.txt")
        assert open(f"{td}/dst.txt").read() == "hello"


def test_copy_tree():
    with tempfile.TemporaryDirectory() as td:
        os.makedirs(f"{td}/src/sub")
        open(f"{td}/src/a.txt", "w").write("a")
        open(f"{td}/src/sub/b.txt", "w").write("b")
        file.copy_tree(f"{td}/src", f"{td}/dst")
        assert open(f"{td}/dst/a.txt").read() == "a"
        assert open(f"{td}/dst/sub/b.txt").read() == "b"


def test_replace():
    with tempfile.TemporaryDirectory() as td:
        open(f"{td}/test.txt", "w").write("hello original world")
        file.replace(f"{td}/test.txt", "original", "REPLACED")
        assert "REPLACED" in open(f"{td}/test.txt").read()


def test_is_empty():
    assert file.is_empty(None) is True
    assert file.is_empty("") is True
    assert file.is_empty("   ") is True
    assert file.is_empty("hello") is False
