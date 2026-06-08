# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_count_index_remove_crashes"
# subject = "cpython.test_list.ListTest.test_count_index_remove_crashes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_count_index_remove_crashes
"""Auto-ported test: ListTest::test_count_index_remove_crashes (CPython 3.12 oracle)."""


import sys
from test import list_tests
from test.support import cpython_only
import pickle
import unittest


# --- test body ---
type2test = list

class X:

    def __eq__(self, other):
        lst.clear()
        return NotImplemented
lst = [X()]
try:
    lst.index(lst)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

class L(list):

    def __eq__(self, other):
        str(other)
        return NotImplemented
lst = L([X()])
lst.count(lst)
lst = L([X()])
try:
    lst.remove(lst)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
lst = [X(), X()]
3 in lst
lst = [X(), X()]
X() in lst
print("ListTest::test_count_index_remove_crashes: ok")
