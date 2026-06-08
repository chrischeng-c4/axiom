# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_convert"
# subject = "cpython.test.test_bool.BoolTest.test_convert"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_convert
"""Auto-ported test: BoolTest::test_convert (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

try:
    bool(42, 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert bool(10) is True

assert bool(1) is True

assert bool(-1) is True

assert bool(0) is False

assert bool('hello') is True

assert bool('') is False

assert bool() is False
print("BoolTest::test_convert: ok")
