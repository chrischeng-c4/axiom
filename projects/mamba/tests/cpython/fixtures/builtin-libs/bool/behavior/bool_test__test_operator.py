# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_operator"
# subject = "cpython.test.test_bool.BoolTest.test_operator"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_operator
"""Auto-ported test: BoolTest::test_operator (CPython 3.12 oracle)."""


import unittest
from test.support import os_helper
import os


# --- test body ---
import operator

assert operator.truth(0) is False

assert operator.truth(1) is True

assert operator.not_(1) is False

assert operator.not_(0) is True

assert operator.contains([], 1) is False

assert operator.contains([1], 1) is True

assert operator.lt(0, 0) is False

assert operator.lt(0, 1) is True

assert operator.is_(True, True) is True

assert operator.is_(True, False) is False

assert operator.is_not(True, True) is False

assert operator.is_not(True, False) is True
print("BoolTest::test_operator: ok")
