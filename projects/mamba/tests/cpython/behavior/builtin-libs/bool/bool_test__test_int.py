# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_int"
# subject = "cpython.test.test_bool.BoolTest.test_int"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_int
"""Auto-ported test: BoolTest::test_int (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert int(False) == 0

assert int(False) is not False

assert int(True) == 1

assert int(True) is not True
print("BoolTest::test_int: ok")
