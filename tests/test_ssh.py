"""Tests for SSH session redaction logic."""

from cutip_blocks.blocks.ssh import SSHSession


def test_redact_password_in_command():
    sesh = SSHSession(
        host="10.0.0.1",
        username="root",
        password="SuperSecret123",
    )
    result = sesh._redact_str("echo SuperSecret123")
    assert "SuperSecret123" not in result
    assert "****" in result


def test_redact_preserves_non_secret_text():
    sesh = SSHSession(
        host="10.0.0.1",
        username="root",
        password="mysecret",
    )
    result = sesh._redact_str("kubectl get deployment -n sfm-1")
    assert result == "kubectl get deployment -n sfm-1"


def test_redact_empty_password():
    sesh = SSHSession(
        host="10.0.0.1",
        username="root",
        password="",
    )
    result = sesh._redact_str("some command")
    assert result == "some command"


def test_redact_multiple_occurrences():
    sesh = SSHSession(
        host="10.0.0.1",
        username="root",
        password="pass123",
    )
    result = sesh._redact_str("echo pass123 && echo pass123")
    assert result == "echo **** && echo ****"
