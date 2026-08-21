# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_09"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_09"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_09
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_09 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
if True and (spam := True):

    assert spam
else:

    raise AssertionError('variable was not assigned using named expression')
print("NamedExpressionAssignmentTest::test_named_expression_assignment_09: ok")
