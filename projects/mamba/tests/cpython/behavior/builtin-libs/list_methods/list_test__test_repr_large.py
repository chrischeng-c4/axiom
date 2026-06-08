# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_repr_large"
# subject = "cpython.test_list.ListTest.test_repr_large"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_repr_large
"""Auto-ported test: ListTest::test_repr_large (CPython 3.12 oracle)."""


def check(n):
    values = [0] * n
    rendered = repr(values)
    assert rendered == "[" + ", ".join(["0"] * n) + "]"


check(10)
check(1_000_000)

print("ListTest::test_repr_large: ok")
