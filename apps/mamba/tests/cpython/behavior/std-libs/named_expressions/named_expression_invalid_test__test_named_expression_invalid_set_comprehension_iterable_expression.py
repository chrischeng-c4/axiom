# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_invalid_test__test_named_expression_invalid_set_comprehension_iterable_expression"
# subject = "cpython.test_named_expressions.NamedExpressionInvalidTest.test_named_expression_invalid_set_comprehension_iterable_expression"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionInvalidTest::test_named_expression_invalid_set_comprehension_iterable_expression
"""Auto-ported test: NamedExpressionInvalidTest::test_named_expression_invalid_set_comprehension_iterable_expression (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
cases = [('Top level', '{i for i in (i := range(5))}'), ('Inside tuple', '{i for i in (2, 3, i := range(5))}'), ('Inside list', '{i for i in {2, 3, i := range(5)}}'), ('Different name', '{i for i in (j := range(5))}'), ('Lambda expression', '{i for i in (lambda:(j := range(5)))()}'), ('Inner loop', '{i for i in range(5) for j in (i := range(5))}'), ('Nested comprehension', '{i for i in {j for j in (k := range(5))}}'), ('Nested comprehension condition', '{i for i in {j for j in range(5) if (j := True)}}'), ('Nested comprehension body', '{i for i in {(j := True) for j in range(5)}}')]
msg = 'assignment expression cannot be used in a comprehension iterable expression'
for case, code in cases:
    try:
        exec(code, {})
        raise AssertionError('expected SyntaxError')
    except SyntaxError as _aR_e:
        import re as _re_aR
        assert _re_aR.search(msg, str(_aR_e))
    try:
        exec(code, {}, {})
        raise AssertionError('expected SyntaxError')
    except SyntaxError as _aR_e:
        import re as _re_aR
        assert _re_aR.search(msg, str(_aR_e))
    try:
        exec(f'lambda: {code}', {})
        raise AssertionError('expected SyntaxError')
    except SyntaxError as _aR_e:
        import re as _re_aR
        assert _re_aR.search(msg, str(_aR_e))
print("NamedExpressionInvalidTest::test_named_expression_invalid_set_comprehension_iterable_expression: ok")
