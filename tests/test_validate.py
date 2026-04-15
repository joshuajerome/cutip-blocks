"""Tests for validate blocks."""

import os
from rsty import validate


def test_path_exists():
    assert validate.path_exists("/tmp") is True
    assert validate.path_exists("/nonexistent/path/xyz") is False


def test_env_var_set():
    os.environ["CUTIP_TEST_VAR"] = "hello"
    assert validate.env_var_set("CUTIP_TEST_VAR") is True
    del os.environ["CUTIP_TEST_VAR"]
    assert validate.env_var_set("CUTIP_TEST_VAR") is False


def test_ip_valid():
    assert validate.ip_valid("192.168.1.1") is True
    assert validate.ip_valid("::1") is True
    assert validate.ip_valid("not-an-ip") is False
    assert validate.ip_valid("") is False
