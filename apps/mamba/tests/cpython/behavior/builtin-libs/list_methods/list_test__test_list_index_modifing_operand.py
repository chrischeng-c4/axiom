# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_list_index_modifing_operand"
# subject = "cpython.test_list.ListTest.test_list_index_modifing_operand"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_list_index_modifing_operand
"""Auto-ported test: ListTest::test_list_index_modifing_operand (CPython 3.12 oracle)."""


import sys
from test import list_tests
from test.support import cpython_only
import pickle
import unittest


# --- test body ---
type2test = list

class evil:

    def __init__(self, lst):
        self.lst = lst

    def __iter__(self):
        yield from self.lst
        self.lst.clear()
lst = list(range(5))
operand = evil(lst)
try:
    lst[::-1] = operand
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("ListTest::test_list_index_modifing_operand: ok")
