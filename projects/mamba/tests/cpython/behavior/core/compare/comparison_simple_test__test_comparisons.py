# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_simple_test__test_comparisons"
# subject = "cpython.test_compare.ComparisonSimpleTest.test_comparisons"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compare.py::ComparisonSimpleTest::test_comparisons
"""Auto-ported test: ComparisonSimpleTest::test_comparisons (CPython 3.12 oracle)."""


class Empty:
    def __repr__(self):
        return "<Empty>"


class Cmp:
    def __init__(self, arg):
        self.arg = arg

    def __repr__(self):
        return f"<Cmp {self.arg}>"

    def __eq__(self, other):
        return self.arg == other


set1 = [2, 2.0, 2, 2 + 0j, Cmp(2.0)]
set2 = [[1], (3,), None, Empty()]
candidates = set1 + set2

for a in candidates:
    for b in candidates:
        if ((a in set1) and (b in set1)) or a is b:
            assert a == b, (a, b)
        else:
            assert a != b, (a, b)

print("ComparisonSimpleTest::test_comparisons: ok")
