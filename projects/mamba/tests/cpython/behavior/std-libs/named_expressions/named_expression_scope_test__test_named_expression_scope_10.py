# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_10"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_10"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_10
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_10 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
res = [(b := [(a := 1) for i in range(2)]) for j in range(2)]

assert res == [[1, 1], [1, 1]]

assert a == 1

assert b == [1, 1]
print("NamedExpressionScopeTest::test_named_expression_scope_10: ok")
