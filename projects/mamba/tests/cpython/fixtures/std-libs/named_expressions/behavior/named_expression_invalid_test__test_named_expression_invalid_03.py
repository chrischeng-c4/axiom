# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_invalid_test__test_named_expression_invalid_03"
# subject = "cpython.test_named_expressions.NamedExpressionInvalidTest.test_named_expression_invalid_03"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionInvalidTest::test_named_expression_invalid_03
"""Auto-ported test: NamedExpressionInvalidTest::test_named_expression_invalid_03 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
code = 'y := f(x)'
try:
    exec(code, {}, {})
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('invalid syntax', str(_aR_e))
print("NamedExpressionInvalidTest::test_named_expression_invalid_03: ok")
