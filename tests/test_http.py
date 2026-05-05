"""Tests for Rust-backed HTTP operations."""

from rsty import http


def test_get():
    resp = http.get("https://httpbin.org/get")
    assert resp.ok
    assert resp.status_code == 200
    data = resp.json()
    assert "url" in data


def test_get_response_headers():
    resp = http.get("https://httpbin.org/get")
    assert resp.ok
    headers = resp.headers
    assert isinstance(headers, dict)
    # Header keys must be lowercase per the response API contract.
    assert all(k == k.lower() for k in headers)
    assert "content-type" in headers


def test_post():
    resp = http.post("https://httpbin.org/post", body="hello")
    assert resp.ok
    assert resp.status_code == 200


def test_post_form():
    resp = http.post(
        "https://httpbin.org/post",
        form={"user": "alice", "lang": "en"},
    )
    assert resp.ok
    data = resp.json()
    assert data["form"] == {"user": "alice", "lang": "en"}
    # httpbin echoes the request Content-Type back under headers.
    assert "application/x-www-form-urlencoded" in data["headers"].get("Content-Type", "")


def test_post_json_still_works():
    resp = http.post("https://httpbin.org/post", json={"k": "v"})
    assert resp.ok
    data = resp.json()
    assert data["json"] == {"k": "v"}


def test_session_basic_get():
    with http.session() as sesh:
        resp = sesh.get("https://httpbin.org/get")
        assert resp.ok
        assert resp.status_code == 200


def test_session_persists_cookies():
    # /cookies/set redirects to /cookies, which echoes back the cookie jar.
    with http.session() as sesh:
        resp = sesh.get("https://httpbin.org/cookies/set?flavor=chocolate")
        assert resp.ok
        # After redirect-following, cookies set on /cookies/set should
        # have been sent back on the /cookies request.
        body = resp.json()
        assert body["cookies"].get("flavor") == "chocolate"


def test_session_no_redirect_exposes_location():
    with http.session(follow_redirects=False) as sesh:
        resp = sesh.get("https://httpbin.org/redirect-to?url=/get")
        assert 300 <= resp.status_code < 400
        assert resp.headers.get("location") == "/get"


def test_session_default_headers():
    with http.session(headers={"X-Test-Default": "yes"}) as sesh:
        resp = sesh.get("https://httpbin.org/headers")
        assert resp.ok
        body = resp.json()
        assert body["headers"].get("X-Test-Default") == "yes"


def test_session_per_call_headers_override_defaults():
    with http.session(headers={"X-Test-Default": "yes"}) as sesh:
        resp = sesh.get(
            "https://httpbin.org/headers",
            headers={"X-Test-Default": "no"},
        )
        assert resp.ok
        body = resp.json()
        assert body["headers"].get("X-Test-Default") == "no"


def test_session_close_then_use_raises():
    sesh = http.session()
    sesh.close()
    try:
        sesh.get("https://httpbin.org/get")
    except RuntimeError as e:
        assert "closed" in str(e).lower()
    else:
        raise AssertionError("expected RuntimeError after close")


def test_imports():
    assert http.get is not None
    assert http.post is not None
    assert http.put is not None
    assert http.delete is not None
    assert http.session is not None
    assert http.HttpResponse is not None
    assert http.HttpSession is not None
