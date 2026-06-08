# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_21"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_21"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_21
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_21 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
def spam(a, b):
    return a + b
res = spam((c := 2), b=1)

assert res == 3

assert c == 2
print("NamedExpressionScopeTest::test_named_expression_scope_21: ok")
