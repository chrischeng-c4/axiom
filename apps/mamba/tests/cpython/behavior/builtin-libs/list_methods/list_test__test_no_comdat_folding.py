# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_no_comdat_folding"
# subject = "cpython.test_list.ListTest.test_no_comdat_folding"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_no_comdat_folding
"""Auto-ported test: ListTest::test_no_comdat_folding (CPython 3.12 oracle)."""


import sys
from test import list_tests
from test.support import cpython_only
import pickle
import unittest


# --- test body ---
type2test = list

class L(list):
    pass
try:
    (3,) + L([1, 2])
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ListTest::test_no_comdat_folding: ok")
