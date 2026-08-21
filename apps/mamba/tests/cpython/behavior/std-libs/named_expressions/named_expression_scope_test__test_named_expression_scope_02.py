# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_02"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_02"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_02
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_02 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
total = 0
partial_sums = [(total := (total + v)) for v in range(5)]

assert partial_sums == [0, 1, 3, 6, 10]

assert total == 10
print("NamedExpressionScopeTest::test_named_expression_scope_02: ok")
