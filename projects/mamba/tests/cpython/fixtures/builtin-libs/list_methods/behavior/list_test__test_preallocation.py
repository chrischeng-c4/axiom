# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_preallocation"
# subject = "cpython.test_list.ListTest.test_preallocation"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_preallocation
"""Auto-ported test: ListTest::test_preallocation (CPython 3.12 oracle)."""


import sys
from test import list_tests
from test.support import cpython_only
import pickle
import unittest


# --- test body ---
type2test = list
iterable = [0] * 10
iter_size = sys.getsizeof(iterable)

assert iter_size == sys.getsizeof(list([0] * 10))

assert iter_size == sys.getsizeof(list(range(10)))
print("ListTest::test_preallocation: ok")
