# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unary"
# dimension = "behavior"
# case = "unary_op_test_case__test_positive"
# subject = "cpython.test_unary.UnaryOpTestCase.test_positive"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unary.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_unary.py::UnaryOpTestCase::test_positive
"""Auto-ported test: UnaryOpTestCase::test_positive (CPython 3.12 oracle)."""


import unittest


'Test compiler changes for unary ops (+, -, ~) introduced in Python 2.2'


# --- test body ---

assert +2 == 2

assert +0 == 0

assert ++2 == 2

assert +2.0 == 2.0

assert +2j == 2j
print("UnaryOpTestCase::test_positive: ok")
