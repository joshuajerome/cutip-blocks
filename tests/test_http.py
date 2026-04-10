"""Tests for Rust-backed HTTP operations."""

from cutip_blocks import http


def test_get():
    resp = http.get("https://httpbin.org/get")
    assert resp.ok
    assert resp.status_code == 200
    data = resp.json()
    assert "url" in data


def test_post():
    resp = http.post("https://httpbin.org/post", body="hello")
    assert resp.ok
    assert resp.status_code == 200


def test_imports():
    assert http.get is not None
    assert http.post is not None
    assert http.put is not None
    assert http.delete is not None
    assert http.HttpResponse is not None
