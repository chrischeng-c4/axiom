# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_simple_test__test_ne_high_priority_uc2879e4"
# subject = "cpython.test_compare.ComparisonSimpleTest.test_ne_high_priority"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
calls = []


class Left:
    def __eq__(*args):
        calls.append("Left.__eq__")
        return NotImplemented


class Right:
    def __eq__(*args):
        calls.append("Right.__eq__")
        return NotImplemented

    def __ne__(*args):
        calls.append("Right.__ne__")
        return NotImplemented


Left() != Right()
assert calls == ["Left.__eq__", "Right.__ne__"], calls

print("ComparisonSimpleTest::test_ne_high_priority: ok")
