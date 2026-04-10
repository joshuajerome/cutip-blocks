"""Tests for config blocks."""

from cutip_blocks import config


def test_render_template():
    result = config.render_template(
        "Hello {{ name }}, port {{ port }}",
        {"name": "world", "port": "8080"},
    )
    assert result == "Hello world, port 8080"


def test_render_template_no_spaces():
    result = config.render_template(
        "Hello {{name}}",
        {"name": "world"},
    )
    assert result == "Hello world"


def test_substitute_vars():
    result = config.substitute_vars(
        "host={{ vars.ip }}, pass={{ secrets.pw }}",
        {"ip": "10.0.0.1"},
        {"pw": "secret123"},
    )
    assert result == "host=10.0.0.1, pass=secret123"


def test_substitute_vars_no_secrets():
    result = config.substitute_vars(
        "host={{ vars.ip }}",
        {"ip": "10.0.0.1"},
    )
    assert result == "host=10.0.0.1"
