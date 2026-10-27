import pytest
from diggity import coalesce, coalesce_logical, dig, dig_path


def test_dig():
    # Test nested dictionary
    data = {"a": {"b": [None, None, {"c": 42}]}}
    assert dig(data, "a", "b", 2, "c") == 42

    # Test missing key with default value
    assert dig(data, "a", "x", default=0) == 0

    # Test missing key without default value
    assert dig(data, "a", "x") is None

    # Test empty args
    assert dig(data) == data


def test_dig_path():
    # Test nested dictionary with default separator
    data = [{"a": {"b": {"c": 42}}}]
    assert dig_path(data, "0.a.b.c") == 42

    # Test nested dictionary with custom separator
    assert dig_path(data, "0/a/b/c", sep="/") == 42

    # Test missing path with default value
    assert dig_path(data, "0.a.x.y", default=0) == 0

    # Test missing path without default value
    assert dig_path(data, "0.a.x.y") is None

    # Test empty path
    assert dig_path(data, "") == data


def test_coalesce():
    # Test with non-None values
    assert coalesce(None, None, 42, None) == 42

    # Test with all None values
    assert coalesce(None, None, None) is None

    # Test with mixed values
    assert coalesce(None, False, 0, "hello") is False

    # Test with no arguments
    assert coalesce() is None


def test_coalesce_logical():
    # Test with truthy values
    assert coalesce_logical(None, False, 42, 0) == 42

    # Test with all falsy values
    assert coalesce_logical(None, False, 0, "") is None

    # Test with mixed values
    assert coalesce_logical(None, False, "hello", 0) == "hello"

    # Test with no arguments
    assert coalesce_logical() is None


# Edge Cases and Error Handling
def test_dig_edge_cases():
    # Test non-dictionary object
    class TestObject:
        def __init__(self):
            self.a = {"b": 42}

    obj = TestObject()
    assert dig(obj, "a", "b") == 42
    assert dig({"a": 1}, 123) is None


def test_dig_path_edge_cases():
    # Test invalid path separator
    data = {"a": {"b": {"c": 42}}}
    assert dig_path(data, "a-b-c", sep="-") == 42

    # Test numeric keys in path
    data = {"a": {0: {"c": 42}}}
    assert dig_path(data, "a.0.c") == 42
    assert dig_path(data, "a.x.y.z") is None


def test_coalesce_edge_cases():
    # Test with empty tuple
    assert coalesce() is None

    # Test with non-None falsy values
    assert coalesce(0, "", False, None) == 0


def test_coalesce_logical_edge_cases():
    # Test with empty tuple
    assert coalesce_logical() is None

    # Test with non-truthy values
    assert coalesce_logical(0, "", False, None) is None


def test_dig_path_negative_index():
    data = [{"a": 1}, {"a": 2}, {"a": 3}]

    assert dig_path(data, "-1.a") == 3
    assert dig_path(data, "-2.a") == 2
    assert dig_path(data, "0.a") == 1

    # Consistent with `dig`, which has always supported negative indices
    assert dig_path(data, "-1.a") == dig(data, -1, "a")

    # Out of range falls back to the default
    assert dig_path(data, "-9.a") is None
    assert dig_path(data, "-9.a", default="DEF") == "DEF"


def test_separator_must_not_be_empty():
    with pytest.raises(ValueError, match="sep"):
        dig_path({"a": 1}, "a", sep="")


def test_dig_propagates_errors_from_user_code():
    class Boom:
        def __getattr__(self, name):
            raise RuntimeError("boom")

    class BoomItem:
        def __getitem__(self, key):
            raise ValueError("boom")

    # A missing key/attribute is still reported via the default value
    assert dig({"a": 1}, "zzz", default="DEF") == "DEF"
    assert dig({"a": 1}, "a", "zzz", default="DEF") == "DEF"
    assert dig([1, 2], "a", default="DEF") == "DEF"

    # ...but genuine errors raised by user code are propagated
    with pytest.raises(RuntimeError, match="boom"):
        dig({"a": Boom()}, "a", "x", default="DEF")
    with pytest.raises(ValueError, match="boom"):
        dig({"a": BoomItem()}, "a", "x", default="DEF")

    with pytest.raises(RuntimeError, match="boom"):
        dig_path({"a": Boom()}, "a.x", default="DEF")
    with pytest.raises(ValueError, match="boom"):
        dig_path({"a": BoomItem()}, "a.x", default="DEF")


def test_fast_paths_preserve_subclass_behaviour():
    """The non-raising fast paths only apply to exact builtin types."""
    from collections import defaultdict

    class MyDict(dict):
        def __missing__(self, key):
            return "fallback"

    class MyList(list):
        def __getitem__(self, key):
            return list.__getitem__(self, key) * 10

    # `__missing__` still runs (an exact dict would have returned the default)
    assert dig({"a": MyDict()}, "a", "zzz") == "fallback"
    assert dig_path({"a": MyDict()}, "a.zzz") == "fallback"

    # `defaultdict` behaves like a normal dict
    dd = defaultdict(int, {"x": 1})
    assert dig({"a": dd}, "a", "zzz") == 0
    assert dig({"a": dd}, "a", "x") == 1

    # overridden `__getitem__` still runs
    assert dig({"a": MyList([1, 2, 3])}, "a", 1) == 20
    assert dig_path({"a": MyList([1, 2, 3])}, "a.1") == 20

    # attribute fallback is still applied to exact dicts and sequences
    assert dig_path({"a": 1}, "keys") is not None
    assert dig([1, 2], "append") is not None


def test_fast_paths_preserve_int_keys():
    # Exact dicts keyed by int must still be reachable through a path segment
    assert dig_path({"a": {0: {"c": 42}}}, "a.0.c") == 42
    assert dig_path({0: "intkey", "0": "strkey"}, "0") == "strkey"

    # sequences keep honouring negative indices
    assert dig_path([{"a": 1}, {"a": 2}], "-1.a") == 2
    assert dig_path([{"a": 1}, {"a": 2}], "-2.a") == 1
    assert dig_path((10, 20, 30), "-1") == 30
    assert dig((10, 20, 30), -1) == 30
    assert dig_path((10, 20, 30), "5") is None
