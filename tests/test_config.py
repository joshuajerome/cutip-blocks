"""Tests for config blocks."""

from rsty import config


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


def test_substitute_vars_paths():
    result = config.substitute_vars(
        "src={{ paths.repo }}",
        {},
        paths={"repo": "/home/u/proj"},
    )
    assert result == "src=/home/u/proj"


def test_substitute_vars_paths_no_spaces():
    result = config.substitute_vars(
        "src={{paths.repo}}",
        {},
        paths={"repo": "/home/u/proj"},
    )
    assert result == "src=/home/u/proj"


def test_substitute_vars_all_three_namespaces():
    result = config.substitute_vars(
        "{{ vars.a }}|{{ paths.b }}|{{ secrets.c }}",
        {"a": "VA"},
        secrets={"c": "SC"},
        paths={"b": "PB"},
    )
    assert result == "VA|PB|SC"


def test_substitute_vars_paths_namespace_isolated():
    # Same key under both vars and paths — each replaces only its own.
    result = config.substitute_vars(
        "v={{ vars.x }} p={{ paths.x }}",
        {"x": "from-vars"},
        paths={"x": "from-paths"},
    )
    assert result == "v=from-vars p=from-paths"


def test_substitute_vars_paths_omitted_leaves_placeholder():
    result = config.substitute_vars("src={{ paths.repo }}", {})
    assert result == "src={{ paths.repo }}"
