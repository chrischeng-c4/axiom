# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_callable"
# subject = "cpython.test.test_bool.BoolTest.test_callable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_callable
"""Auto-ported test: BoolTest::test_callable (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert callable(len) is True

assert callable(1) is False
print("BoolTest::test_callable: ok")
