# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_invalid_test__test_named_expression_invalid_rebinding_iteration_variable"
# subject = "cpython.test_named_expressions.NamedExpressionInvalidTest.test_named_expression_invalid_rebinding_iteration_variable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionInvalidTest::test_named_expression_invalid_rebinding_iteration_variable
"""Auto-ported test: NamedExpressionInvalidTest::test_named_expression_invalid_rebinding_iteration_variable (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
cases = [('Complex expression: a', 'a', '{0}(a := 1) for a, (*b, c[d+e::f(g)], h.i) in j{1}'), ('Complex expression: b', 'b', '{0}(b := 1) for a, (*b, c[d+e::f(g)], h.i) in j{1}')]
for test_case, target, code in cases:
    msg = f"assignment expression cannot rebind comprehension iteration variable '{target}'"
    for lpar, rpar in [('(', ')'), ('[', ']'), ('{', '}')]:
        code = code.format(lpar, rpar)
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
print("NamedExpressionInvalidTest::test_named_expression_invalid_rebinding_iteration_variable: ok")
