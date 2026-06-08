# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_19"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_19"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_19
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_19 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
def spam(a):
    return a
res = spam((b := 2))

assert res == 2

assert b == 2
print("NamedExpressionScopeTest::test_named_expression_scope_19: ok")
