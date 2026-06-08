# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_float"
# subject = "cpython.test.test_bool.BoolTest.test_float"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_float
"""Auto-ported test: BoolTest::test_float (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert float(False) == 0.0

assert float(False) is not False

assert float(True) == 1.0

assert float(True) is not True
print("BoolTest::test_float: ok")
