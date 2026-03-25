"""Tests for BlockRegistry."""

from cutip_blocks.registry import BlockRegistry


def test_discover_finds_all_blocks():
    registry = BlockRegistry.discover()
    assert len(registry.blocks) > 0


def test_discover_finds_expected_categories():
    registry = BlockRegistry.discover()
    cats = registry.categories()
    assert "container" in cats
    assert "ssh" in cats
    assert "file" in cats
    assert "k8s" in cats
    assert "crictl" in cats


def test_get_by_category_and_action():
    registry = BlockRegistry.discover()
    result = registry.get("k8s", "get_secret")
    assert result is not None
    meta, fn = result
    assert meta.name == "Get Secret"
    assert meta.category == "k8s"
    assert meta.action == "get_secret"


def test_get_returns_none_for_unknown():
    registry = BlockRegistry.discover()
    assert registry.get("nonexistent", "nope") is None


def test_no_duplicate_blocks():
    registry = BlockRegistry.discover()
    keys = [(m.category, m.action) for m, _ in registry.blocks]
    assert len(keys) == len(set(keys)), f"Duplicate blocks: {keys}"
