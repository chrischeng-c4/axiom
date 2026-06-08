# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_issubclass"
# subject = "cpython.test.test_bool.BoolTest.test_issubclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_issubclass
"""Auto-ported test: BoolTest::test_issubclass (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert issubclass(bool, int) is True

assert issubclass(int, bool) is False
print("BoolTest::test_issubclass: ok")
