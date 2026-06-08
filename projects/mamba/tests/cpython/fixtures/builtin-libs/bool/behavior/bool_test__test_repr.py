# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_repr"
# subject = "cpython.test.test_bool.BoolTest.test_repr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_repr
"""Auto-ported test: BoolTest::test_repr (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---

assert repr(False) == 'False'

assert repr(True) == 'True'

assert eval(repr(False)) is False

assert eval(repr(True)) is True
print("BoolTest::test_repr: ok")
