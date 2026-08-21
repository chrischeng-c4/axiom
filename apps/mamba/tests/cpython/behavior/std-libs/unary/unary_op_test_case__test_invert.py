# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unary"
# dimension = "behavior"
# case = "unary_op_test_case__test_invert"
# subject = "cpython.test_unary.UnaryOpTestCase.test_invert"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unary.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_unary.py::UnaryOpTestCase::test_invert
"""Auto-ported test: UnaryOpTestCase::test_invert (CPython 3.12 oracle)."""


import unittest


'Test compiler changes for unary ops (+, -, ~) introduced in Python 2.2'


# --- test body ---

assert ~2 == -(2 + 1)

assert ~0 == -1

assert ~~2 == 2
print("UnaryOpTestCase::test_invert: ok")
