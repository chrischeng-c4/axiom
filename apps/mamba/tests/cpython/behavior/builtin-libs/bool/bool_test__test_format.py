# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_format"
# subject = "cpython.test.test_bool.BoolTest.test_format"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_format
"""Auto-ported test: BoolTest::test_format (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert '%d' % False == '0'

assert '%d' % True == '1'

assert '%x' % False == '0'

assert '%x' % True == '1'
print("BoolTest::test_format: ok")
