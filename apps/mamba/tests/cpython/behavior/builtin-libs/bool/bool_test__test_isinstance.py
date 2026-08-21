# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_isinstance"
# subject = "cpython.test.test_bool.BoolTest.test_isinstance"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_isinstance
"""Auto-ported test: BoolTest::test_isinstance (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert isinstance(True, bool) is True

assert isinstance(False, bool) is True

assert isinstance(True, int) is True

assert isinstance(False, int) is True

assert isinstance(1, bool) is False

assert isinstance(0, bool) is False
print("BoolTest::test_isinstance: ok")
