# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_full_test__test_mappings_uc8f876c"
# subject = "cpython.test_compare.ComparisonFullTest.test_mappings"
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


d1 = {1: "a", 2: "b"}
d2 = {2: "b", 3: "c"}
d3 = {3: "c", 2: "b"}

assert_equality_only(d1, d1, True)
assert_equality_only(d1, d2, False)
assert_equality_only(d2, d3, True)

print("ComparisonFullTest::test_mappings: ok")
