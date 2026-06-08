# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_01"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_01"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_01
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_01 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
code = 'def spam():\n    (a := 5)\nprint(a)'
try:
    exec(code, {}, {})
    raise AssertionError('expected NameError')
except NameError as _aR_e:
    import re as _re_aR
    assert _re_aR.search("name 'a' is not defined", str(_aR_e))
print("NamedExpressionScopeTest::test_named_expression_scope_01: ok")
