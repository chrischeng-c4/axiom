# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_real_and_imag"
# subject = "cpython.test.test_bool.BoolTest.test_real_and_imag"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_real_and_imag
"""Auto-ported test: BoolTest::test_real_and_imag (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert True .real == 1

assert True .imag == 0

assert type(True .real) is int

assert type(True .imag) is int

assert False .real == 0

assert False .imag == 0

assert type(False .real) is int

assert type(False .imag) is int
print("BoolTest::test_real_and_imag: ok")
