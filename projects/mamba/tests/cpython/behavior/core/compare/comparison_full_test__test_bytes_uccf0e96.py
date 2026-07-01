# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_full_test__test_bytes_uccf0e96"
# subject = "cpython.test_compare.ComparisonFullTest.test_bytes"
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


bs1 = b"a1"
bs2 = b"b2"
assert_total_order(bs1, bs1, 0)
assert_total_order(bs1, bs2, -1)

ba1 = bytearray(b"a1")
ba2 = bytearray(b"b2")
assert_total_order(ba1, ba1, 0)
assert_total_order(ba1, ba2, -1)

assert_total_order(bs1, ba1, 0)
assert_total_order(bs1, ba2, -1)
assert_total_order(ba1, bs1, 0)
assert_total_order(ba1, bs2, -1)

print("ComparisonFullTest::test_bytes: ok")
