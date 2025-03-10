from diggity import dig, dig_path, coalesce, coalesce_logical


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
    assert coalesce(None, False, 0, "hello") == False

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
