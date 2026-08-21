# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_bool_called_at_least_once"
# subject = "cpython.test.test_bool.BoolTest.test_bool_called_at_least_once"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_bool_called_at_least_once
"""Auto-ported test: BoolTest::test_bool_called_at_least_once (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---
class X:

    def __init__(self):
        self.count = 0

    def __bool__(self):
        self.count += 1
        return True

def f(x):
    if x or True:
        pass
x = X()
f(x)

assert x.count >= 1
print("BoolTest::test_bool_called_at_least_once: ok")
