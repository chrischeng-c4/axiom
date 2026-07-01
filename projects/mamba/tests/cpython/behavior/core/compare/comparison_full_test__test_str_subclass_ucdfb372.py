# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_full_test__test_str_subclass_ucdfb372"
# subject = "cpython.test_compare.ComparisonFullTest.test_str_subclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
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


class StrSubclass(str):
    pass


s1 = str("a")
s2 = str("b")
c1 = StrSubclass("a")
c2 = StrSubclass("b")
c3 = StrSubclass("b")

assert_total_order(s1, s1, 0)
assert_total_order(s1, s2, -1)
assert_total_order(c1, c1, 0)
assert_total_order(c1, c2, -1)
assert_total_order(c2, c3, 0)

assert_total_order(s1, c2, -1)
assert_total_order(s2, c3, 0)
assert_total_order(c1, s2, -1)
assert_total_order(c2, s2, 0)

print("ComparisonFullTest::test_str_subclass: ok")
