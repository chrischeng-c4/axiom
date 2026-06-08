# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_blocked"
# subject = "cpython.test.test_bool.BoolTest.test_blocked"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_blocked
"""Auto-ported test: BoolTest::test_blocked (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---
class A:
    __bool__ = None

try:
    bool(A())
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class B:

    def __len__(self):
        return 10
    __bool__ = None

try:
    bool(B())
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("BoolTest::test_blocked: ok")
