# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_simple_test__test_ne_defaults_to_not_eq_uc0267c8"
# subject = "cpython.test_compare.ComparisonSimpleTest.test_ne_defaults_to_not_eq"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
class Cmp:
    def __init__(self, arg):
        self.arg = arg

    def __eq__(self, other):
        return self.arg == other.arg


a = Cmp(1)
b = Cmp(1)
c = Cmp(2)
assert (a == b) is True
assert (a != b) is False
assert (a != c) is True

print("ComparisonSimpleTest::test_ne_defaults_to_not_eq: ok")
