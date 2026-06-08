# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_len"
# subject = "cpython.test_list.ListTest.test_len"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_len
"""Auto-ported test: ListTest::test_len (CPython 3.12 oracle)."""


type2test = list

assert len(type2test()) == 0
assert len(type2test([0])) == 1
assert len(type2test([0, 1, 2])) == 3
assert len([]) == 0
assert len([0]) == 1
assert len([0, 1, 2]) == 3

print("ListTest::test_len: ok")
