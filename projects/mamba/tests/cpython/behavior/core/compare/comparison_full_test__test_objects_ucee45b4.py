# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_full_test__test_objects_ucee45b4"
# subject = "cpython.test_compare.ComparisonFullTest.test_objects"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
def expect_type_error(func):
    try:
        func()
    except TypeError as exc:
        assert "not supported" in str(exc)
        return
    raise AssertionError("ordering operation did not raise TypeError")


def assert_equality_only(a, b, equal):
    assert (a == b) == equal
    assert (b == a) == equal
    assert (a != b) == (not equal)
    assert (b != a) == (not equal)

    expect_type_error(lambda: a < b)
    expect_type_error(lambda: a <= b)
    expect_type_error(lambda: a > b)
    expect_type_error(lambda: a >= b)
    expect_type_error(lambda: b < a)
    expect_type_error(lambda: b <= a)
    expect_type_error(lambda: b > a)
    expect_type_error(lambda: b >= a)


a = object()
b = object()
assert_equality_only(a, a, True)
assert_equality_only(a, b, False)

print("ComparisonFullTest::test_objects: ok")
