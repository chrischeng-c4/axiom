# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_contains"
# subject = "cpython.test.test_bool.BoolTest.test_contains"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_contains
"""Auto-ported test: BoolTest::test_contains (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert (1 in {}) is False

assert (1 in {1: 1}) is True
print("BoolTest::test_contains: ok")
