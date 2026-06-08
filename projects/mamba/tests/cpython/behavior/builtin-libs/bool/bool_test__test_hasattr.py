# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_hasattr"
# subject = "cpython.test.test_bool.BoolTest.test_hasattr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_hasattr
"""Auto-ported test: BoolTest::test_hasattr (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert hasattr([], 'append') is True

assert hasattr([], 'wobble') is False
print("BoolTest::test_hasattr: ok")
