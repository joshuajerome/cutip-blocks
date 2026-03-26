"""Tests for @block decorator and BlockMeta."""

from cutip_blocks.decorator import BlockMeta, _BLOCK_ATTR, block


def test_block_decorator_attaches_meta():
    @block(name="Test Block", category="test", action="do_thing")
    def my_block(ctx):
        return "ok"

    meta = getattr(my_block, _BLOCK_ATTR)
    assert isinstance(meta, BlockMeta)
    assert meta.name == "Test Block"
    assert meta.category == "test"
    assert meta.action == "do_thing"


def test_block_preserves_function_behavior():
    @block(name="Add", category="math", action="add")
    def add(a, b):
        return a + b

    assert add(2, 3) == 5


def test_block_preserves_function_name():
    @block(name="My Func", category="test", action="my_func")
    def my_func():
        pass

    assert my_func.__name__ == "my_func"


def test_block_meta_is_frozen():
    meta = BlockMeta(name="X", category="y", action="z")
    try:
        meta.name = "changed"
        raise AssertionError("Should have raised")
    except AttributeError:
        pass
