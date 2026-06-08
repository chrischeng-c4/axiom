# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unary"
# dimension = "behavior"
# case = "unary_op_test_case__test_no_overflow"
# subject = "cpython.test_unary.UnaryOpTestCase.test_no_overflow"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unary.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unary.py::UnaryOpTestCase::test_no_overflow
"""Auto-ported test: UnaryOpTestCase::test_no_overflow (CPython 3.12 oracle)."""


import unittest


'Test compiler changes for unary ops (+, -, ~) introduced in Python 2.2'


# --- test body ---
nines = '9' * 32

assert eval('+' + nines) == 10 ** 32 - 1

assert eval('-' + nines) == -(10 ** 32 - 1)

assert eval('~' + nines) == ~(10 ** 32 - 1)
print("UnaryOpTestCase::test_no_overflow: ok")
