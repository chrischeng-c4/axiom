# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_14"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_14"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_14
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_14 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
"""
        Where all variables are positive integers, and a is at least as large
        as the n'th root of x, this algorithm returns the floor of the n'th
        root of x (and roughly doubling the number of accurate bits per
        iteration):
        """
a = 9
n = 2
x = 3
while a > (d := (x // a ** (n - 1))):
    a = ((n - 1) * a + d) // n

assert a == 1
print("NamedExpressionAssignmentTest::test_named_expression_assignment_14: ok")
