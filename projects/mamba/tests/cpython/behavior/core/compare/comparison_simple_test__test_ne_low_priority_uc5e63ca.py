# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_simple_test__test_ne_low_priority_uc5e63ca"
# subject = "cpython.test_compare.ComparisonSimpleTest.test_ne_low_priority"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
calls = []


class Base:
    def __eq__(*args):
        calls.append("Base.__eq__")
        return NotImplemented


class Derived(Base):
    def __eq__(*args):
        calls.append("Derived.__eq__")
        return NotImplemented

    def __ne__(*args):
        calls.append("Derived.__ne__")
        return NotImplemented


Base() != Derived()
assert calls == ["Derived.__ne__", "Base.__eq__"], calls

print("ComparisonSimpleTest::test_ne_low_priority: ok")
