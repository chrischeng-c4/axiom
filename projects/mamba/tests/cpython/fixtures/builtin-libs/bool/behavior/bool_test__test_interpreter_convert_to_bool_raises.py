# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_interpreter_convert_to_bool_raises"
# subject = "cpython.test.test_bool.BoolTest.test_interpreter_convert_to_bool_raises"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_interpreter_convert_to_bool_raises
"""Auto-ported test: BoolTest::test_interpreter_convert_to_bool_raises (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---
class SymbolicBool:

    def __bool__(self):
        raise TypeError

class Symbol:

    def __gt__(self, other):
        return SymbolicBool()
x = Symbol()
try:
    if x > 0:
        msg = 'x > 0 was true'
    else:
        msg = 'x > 0 was false'
    raise AssertionError('expected TypeError')
except TypeError:
    pass
del x
print("BoolTest::test_interpreter_convert_to_bool_raises: ok")
