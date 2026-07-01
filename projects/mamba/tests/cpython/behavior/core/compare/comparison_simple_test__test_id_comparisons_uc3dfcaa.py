# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_simple_test__test_id_comparisons_uc3dfcaa"
# subject = "cpython.test_compare.ComparisonSimpleTest.test_id_comparisons"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
class Empty:
    def __repr__(self):
        return "<Empty>"


items = []
for i in range(10):
    items.insert(len(items) // 2, Empty())

for a in items:
    for b in items:
        assert (a == b) == (a is b), "a=%r, b=%r" % (a, b)

print("ComparisonSimpleTest::test_id_comparisons: ok")
