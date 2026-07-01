# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_full_test__test_numbers_ucdaaeb5"
# subject = "cpython.test_compare.ComparisonFullTest.test_numbers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
from decimal import Decimal
from fractions import Fraction


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


i1 = 1001
i2 = 1002
assert_total_order(i1, i1, 0)
assert_total_order(i1, i2, -1)

f1 = 1001.0
f2 = 1001.1
assert_total_order(f1, f1, 0)
assert_total_order(f1, f2, -1)

q1 = Fraction(2002, 2)
q2 = Fraction(2003, 2)
assert_total_order(q1, q1, 0)
assert_total_order(q1, q2, -1)

d1 = Decimal("1001.0")
d2 = Decimal("1001.1")
assert_total_order(d1, d1, 0)
assert_total_order(d1, d2, -1)

c1 = 1001 + 0j
c2 = 1001 + 1j
assert_equality_only(c1, c1, True)
assert_equality_only(c1, c2, False)

for n1, n2 in ((i1, f1), (i1, q1), (i1, d1), (f1, q1), (f1, d1), (q1, d1)):
    assert_total_order(n1, n2, 0)

for n1 in (i1, f1, q1, d1):
    assert_equality_only(n1, c1, True)

print("ComparisonFullTest::test_numbers: ok")
