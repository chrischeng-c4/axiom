# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_list_resize_overflow"
# subject = "cpython.test_list.ListTest.test_list_resize_overflow"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_list_resize_overflow
"""Auto-ported test: ListTest::test_list_resize_overflow (CPython 3.12 oracle)."""


import sys
from test import list_tests
from test.support import cpython_only
import pickle
import unittest


# --- test body ---
type2test = list
lst = [0] * 65
del lst[1:]

assert len(lst) == 1
size = sys.maxsize
try:
    lst * size
    raise AssertionError('expected (MemoryError, OverflowError)')
except (MemoryError, OverflowError):
    pass
try:
    lst *= size
    raise AssertionError('expected (MemoryError, OverflowError)')
except (MemoryError, OverflowError):
    pass
print("ListTest::test_list_resize_overflow: ok")
