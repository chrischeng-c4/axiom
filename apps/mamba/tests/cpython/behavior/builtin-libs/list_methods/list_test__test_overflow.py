# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_overflow"
# subject = "cpython.test_list.ListTest.test_overflow"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_overflow
"""Auto-ported test: ListTest::test_overflow (CPython 3.12 oracle)."""


import sys
from test import list_tests
from test.support import cpython_only
import pickle
import unittest


# --- test body ---
type2test = list
lst = [4, 5, 6, 7]
n = int((sys.maxsize * 2 + 2) // len(lst))

def mul(a, b):
    return a * b

def imul(a, b):
    a *= b

try:
    mul(lst, n)
    raise AssertionError('expected (MemoryError, OverflowError)')
except (MemoryError, OverflowError):
    pass

try:
    imul(lst, n)
    raise AssertionError('expected (MemoryError, OverflowError)')
except (MemoryError, OverflowError):
    pass
print("ListTest::test_overflow: ok")
