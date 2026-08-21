# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_full_test__test_sequences_ucd89ef3"
# subject = "cpython.test_compare.ComparisonFullTest.test_sequences"
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


def assert_total_order(a, b, comp):
    assert (a == b) == (comp == 0)
    assert (b == a) == (comp == 0)
    assert (a != b) == (comp != 0)
    assert (b != a) == (comp != 0)

    assert (a < b) == (comp < 0)
    assert (a <= b) == (comp <= 0)
    assert (a > b) == (comp > 0)
    assert (a >= b) == (comp >= 0)

    assert (b < a) == (comp > 0)
    assert (b <= a) == (comp >= 0)
    assert (b > a) == (comp < 0)
    assert (b >= a) == (comp <= 0)


l1 = [1, 2]
l2 = [2, 3]
assert_total_order(l1, l1, 0)
assert_total_order(l1, l2, -1)

t1 = (1, 2)
t2 = (2, 3)
assert_total_order(t1, t1, 0)
assert_total_order(t1, t2, -1)

r1 = range(1, 2)
r2 = range(2, 2)
assert_equality_only(r1, r1, True)
assert_equality_only(r1, r2, False)

assert_equality_only(t1, l1, False)
assert_equality_only(l1, r1, False)
assert_equality_only(r1, t1, False)

print("ComparisonFullTest::test_sequences: ok")
