# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_truth"
# subject = "cpython.test_list.ListTest.test_truth"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_truth
"""Auto-ported test: ListTest::test_truth (CPython 3.12 oracle)."""


type2test = list

assert not type2test()
assert type2test([42])
assert not []
assert [42]

print("ListTest::test_truth: ok")
