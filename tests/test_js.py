"""Tests for rsty.js — JS module read/write."""

from __future__ import annotations

from pathlib import Path

import pytest

from rsty import js


SAMPLE_NAMED = """const PROXY_CONFIG = {
  '/redfish': {
    target: 'http://100.94.115.88',
    secure: false,
    logLevel: 'debug',
    auth: 'admin:secret123',
  },
  '/api': {
    target: 'http://100.94.115.88',
    secure: false,
    logLevel: 'debug',
    auth: 'admin:secret123',
  },
};

module.exports = PROXY_CONFIG;
"""


SAMPLE_DIRECT = """module.exports = {
  port: 4000,
  paths: ['/api', '/login'],
};
"""


def test_read_module_named(tmp_path: Path):
    p = tmp_path / "proxy.conf.json"
    p.write_text(SAMPLE_NAMED)
    config = js.read_module(p)
    assert set(config.keys()) == {"/redfish", "/api"}
    assert config["/redfish"]["target"] == "http://100.94.115.88"
    assert config["/redfish"]["secure"] is False
    assert config["/api"]["auth"] == "admin:secret123"


def test_read_module_direct(tmp_path: Path):
    p = tmp_path / "config.js"
    p.write_text(SAMPLE_DIRECT)
    config = js.read_module(p)
    assert config["port"] == 4000
    assert config["paths"] == ["/api", "/login"]


def test_read_module_missing_module_exports(tmp_path: Path):
    p = tmp_path / "bad.js"
    p.write_text("const X = { a: 1 };")
    with pytest.raises(RuntimeError, match="module.exports"):
        js.read_module(p)


def test_round_trip_preserves_keys(tmp_path: Path):
    p = tmp_path / "proxy.conf.json"
    p.write_text(SAMPLE_NAMED)
    config = js.read_module(p)
    js.write_module(p, config)
    again = js.read_module(p)
    assert again == config


def test_write_module_preserves_named_wrapper(tmp_path: Path):
    p = tmp_path / "proxy.conf.json"
    p.write_text(SAMPLE_NAMED)
    config = js.read_module(p)
    config["/redfish"]["target"] = "http://10.0.0.1"
    js.write_module(p, config)
    text = p.read_text()
    # Named wrapper retained
    assert "const PROXY_CONFIG = " in text
    assert "module.exports = PROXY_CONFIG;" in text


def test_write_module_preserves_direct_wrapper(tmp_path: Path):
    p = tmp_path / "config.js"
    p.write_text(SAMPLE_DIRECT)
    config = js.read_module(p)
    config["port"] = 5000
    js.write_module(p, config)
    text = p.read_text()
    assert "const " not in text
    assert text.startswith("module.exports = ")


def test_write_module_new_file_defaults_to_direct(tmp_path: Path):
    p = tmp_path / "new.js"
    js.write_module(p, {"a": 1, "b": [2, 3]})
    text = p.read_text()
    assert text.startswith("module.exports = ")
    again = js.read_module(p)
    assert again == {"a": 1, "b": [2, 3]}


def test_unknown_keys_preserved(tmp_path: Path):
    """Add a 5th per-section key in the file, ensure it survives a round-trip."""
    src = """const PROXY_CONFIG = {
  '/redfish': {
    target: 'http://x',
    secure: false,
    logLevel: 'debug',
    auth: 'u:p',
    customKey: 'some-extra-key',
  },
};

module.exports = PROXY_CONFIG;
"""
    p = tmp_path / "proxy.conf.json"
    p.write_text(src)
    config = js.read_module(p)
    assert config["/redfish"]["customKey"] == "some-extra-key"
    # Mutate target only; customKey untouched
    config["/redfish"]["target"] = "http://10.0.0.1"
    js.write_module(p, config)
    again = js.read_module(p)
    assert again["/redfish"]["customKey"] == "some-extra-key"
    assert again["/redfish"]["target"] == "http://10.0.0.1"


def test_new_section_added_after_read(tmp_path: Path):
    p = tmp_path / "proxy.conf.json"
    p.write_text(SAMPLE_NAMED)
    config = js.read_module(p)
    config["/extra"] = {
        "target": "http://10.0.0.1",
        "secure": True,
        "logLevel": "info",
        "auth": "x:y",
    }
    js.write_module(p, config)
    again = js.read_module(p)
    assert "/extra" in again
    assert again["/extra"]["secure"] is True


def test_comments_in_file_dont_break_parse(tmp_path: Path):
    src = """// top-level comment
const X = {
  /* block comment with } in it */
  a: 1,
  b: 'has "double" inside',
};
module.exports = X;
"""
    p = tmp_path / "config.js"
    p.write_text(src)
    config = js.read_module(p)
    assert config == {"a": 1, "b": 'has "double" inside'}
