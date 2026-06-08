# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_step_overflow"
# subject = "cpython.test_list.ListTest.test_step_overflow"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_step_overflow
"""Auto-ported test: ListTest::test_step_overflow (CPython 3.12 oracle)."""


import sys
from test import list_tests
from test.support import cpython_only
import pickle
import unittest


# --- test body ---
type2test = list
a = [0, 1, 2, 3, 4]
a[1::sys.maxsize] = [0]

assert a[3::sys.maxsize] == [3]
print("ListTest::test_step_overflow: ok")
