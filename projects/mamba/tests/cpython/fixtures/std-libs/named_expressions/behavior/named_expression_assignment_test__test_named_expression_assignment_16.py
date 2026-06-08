# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_16"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_16"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_16
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_16 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
a, b = (1, 2)
fib = {(c := a): (a := b) + (b := (a + c)) - b for __ in range(6)}

assert fib == {1: 2, 2: 3, 3: 5, 5: 8, 8: 13, 13: 21}
print("NamedExpressionAssignmentTest::test_named_expression_assignment_16: ok")
