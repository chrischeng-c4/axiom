# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_13"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_13"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_13
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_13 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
length = len((lines := [1, 2]))

assert length == 2

assert lines == [1, 2]
print("NamedExpressionAssignmentTest::test_named_expression_assignment_13: ok")
