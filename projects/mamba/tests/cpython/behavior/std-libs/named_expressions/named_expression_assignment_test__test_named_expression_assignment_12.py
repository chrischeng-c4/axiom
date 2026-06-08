# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_12"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_12"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_12
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_12 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
def spam(a):
    return a
res = [[(y := spam(x)), x / y] for x in range(1, 5)]

assert res == [[1, 1.0], [2, 1.0], [3, 1.0], [4, 1.0]]
print("NamedExpressionAssignmentTest::test_named_expression_assignment_12: ok")
