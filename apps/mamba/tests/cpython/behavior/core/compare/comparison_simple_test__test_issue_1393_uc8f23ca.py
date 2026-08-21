# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_simple_test__test_issue_1393_uc8f23ca"
# subject = "cpython.test_compare.ComparisonSimpleTest.test_issue_1393"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
class AlwaysEq:
    def __eq__(self, other):
        return True

    def __ne__(self, other):
        return False


always_eq = AlwaysEq()

x = lambda: None
assert x == always_eq
assert always_eq == x

y = object()
assert y == always_eq
assert always_eq == y

print("ComparisonSimpleTest::test_issue_1393: ok")
