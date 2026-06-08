# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_types"
# subject = "cpython.test.test_bool.BoolTest.test_types"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_types
"""Auto-ported test: BoolTest::test_types (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---
for t in [bool, complex, dict, float, int, list, object, set, str, tuple, type]:

    assert bool(t) is True
print("BoolTest::test_types: ok")
