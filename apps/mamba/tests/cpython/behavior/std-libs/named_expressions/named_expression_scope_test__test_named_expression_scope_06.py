# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_06"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_06"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_06
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_06 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
res = [[(spam := i) for i in range(3)] for j in range(2)]

assert res == [[0, 1, 2], [0, 1, 2]]

assert spam == 2
print("NamedExpressionScopeTest::test_named_expression_scope_06: ok")
