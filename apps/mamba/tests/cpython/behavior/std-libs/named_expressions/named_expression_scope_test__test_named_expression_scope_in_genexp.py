# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_in_genexp"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_in_genexp"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_in_genexp
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_in_genexp (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
a = 1
b = [1, 2, 3, 4]
genexp = ((c := (i + a)) for i in b)

assert 'c' not in locals()
for idx, elem in enumerate(genexp):

    assert elem == b[idx] + a
print("NamedExpressionScopeTest::test_named_expression_scope_in_genexp: ok")
