# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_bool_new"
# subject = "cpython.test.test_bool.BoolTest.test_bool_new"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_bool_new
"""Auto-ported test: BoolTest::test_bool_new (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert bool.__new__(bool) is False

assert bool.__new__(bool, 1) is True

assert bool.__new__(bool, 0) is False

assert bool.__new__(bool, False) is False

assert bool.__new__(bool, True) is True
print("BoolTest::test_bool_new: ok")
