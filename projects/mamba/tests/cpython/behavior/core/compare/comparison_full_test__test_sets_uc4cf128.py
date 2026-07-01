# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_full_test__test_sets_uc4cf128"
# subject = "cpython.test_compare.ComparisonFullTest.test_sets"
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


s1 = {1, 2}
s2 = {1, 2, 3}
assert_total_order(s1, s1, 0)
assert_total_order(s1, s2, -1)

f1 = frozenset(s1)
f2 = frozenset(s2)
assert_total_order(f1, f1, 0)
assert_total_order(f1, f2, -1)

assert_total_order(s1, f1, 0)
assert_total_order(s1, f2, -1)
assert_total_order(f1, s1, 0)
assert_total_order(f1, s2, -1)

print("ComparisonFullTest::test_sets: ok")
