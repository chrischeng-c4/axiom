# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_23"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_23"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_23
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_23 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
def spam(a, b):
    return a + b
res = spam(b=(c := 2), a=1)

assert res == 3

assert c == 2
print("NamedExpressionScopeTest::test_named_expression_scope_23: ok")
