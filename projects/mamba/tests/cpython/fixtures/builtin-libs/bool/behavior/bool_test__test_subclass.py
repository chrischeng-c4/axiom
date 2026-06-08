# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_subclass"
# subject = "cpython.test.test_bool.BoolTest.test_subclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_subclass
"""Auto-ported test: BoolTest::test_subclass (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---
try:

    class C(bool):
        pass
except TypeError:
    pass
else:

    raise AssertionError('bool should not be subclassable')

try:
    int.__new__(bool, 0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("BoolTest::test_subclass: ok")
