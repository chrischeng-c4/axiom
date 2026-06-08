# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unary"
# dimension = "behavior"
# case = "unary_op_test_case__test_negation_of_exponentiation"
# subject = "cpython.test_unary.UnaryOpTestCase.test_negation_of_exponentiation"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unary.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unary.py::UnaryOpTestCase::test_negation_of_exponentiation
"""Auto-ported test: UnaryOpTestCase::test_negation_of_exponentiation (CPython 3.12 oracle)."""


import unittest


'Test compiler changes for unary ops (+, -, ~) introduced in Python 2.2'


# --- test body ---

assert -2 ** 3 == -8

assert (-2) ** 3 == -8

assert -2 ** 4 == -16

assert (-2) ** 4 == 16
print("UnaryOpTestCase::test_negation_of_exponentiation: ok")
