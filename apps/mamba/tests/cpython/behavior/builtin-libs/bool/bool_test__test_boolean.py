# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_boolean"
# subject = "cpython.test.test_bool.BoolTest.test_boolean"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_boolean
"""Auto-ported test: BoolTest::test_boolean (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert True & 1 == 1

assert not isinstance(True & 1, bool)

assert True & True is True

assert True | 1 == 1

assert not isinstance(True | 1, bool)

assert True | True is True

assert True ^ 1 == 0

assert not isinstance(True ^ 1, bool)

assert True ^ True is False
print("BoolTest::test_boolean: ok")
