# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_identity"
# subject = "cpython.test_list.ListTest.test_identity"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_identity
"""Auto-ported test: ListTest::test_identity (CPython 3.12 oracle)."""


import sys
from test import list_tests
from test.support import cpython_only
import pickle
import unittest


# --- test body ---
type2test = list

assert [] is not []
print("ListTest::test_identity: ok")
