# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_invalid_test__test_named_expression_valid_rebinding_iteration_variable"
# subject = "cpython.test_named_expressions.NamedExpressionInvalidTest.test_named_expression_valid_rebinding_iteration_variable"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionInvalidTest::test_named_expression_valid_rebinding_iteration_variable
"""Auto-ported test: NamedExpressionInvalidTest::test_named_expression_valid_rebinding_iteration_variable (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
cases = [('Complex expression: c', '{0}(c := 1) for a, (*b, c[d+e::f(g)], h.i) in j{1}'), ('Complex expression: d', '{0}(d := 1) for a, (*b, c[d+e::f(g)], h.i) in j{1}'), ('Complex expression: e', '{0}(e := 1) for a, (*b, c[d+e::f(g)], h.i) in j{1}'), ('Complex expression: f', '{0}(f := 1) for a, (*b, c[d+e::f(g)], h.i) in j{1}'), ('Complex expression: g', '{0}(g := 1) for a, (*b, c[d+e::f(g)], h.i) in j{1}'), ('Complex expression: h', '{0}(h := 1) for a, (*b, c[d+e::f(g)], h.i) in j{1}'), ('Complex expression: i', '{0}(i := 1) for a, (*b, c[d+e::f(g)], h.i) in j{1}'), ('Complex expression: j', '{0}(j := 1) for a, (*b, c[d+e::f(g)], h.i) in j{1}')]
for test_case, code in cases:
    for lpar, rpar in [('(', ')'), ('[', ']'), ('{', '}')]:
        code = code.format(lpar, rpar)
        try:
            exec(code, {})
            raise AssertionError('expected NameError')
        except NameError:
            pass
        try:
            exec(code, {}, {})
            raise AssertionError('expected NameError')
        except NameError:
            pass
        exec(f'lambda: {code}', {})
print("NamedExpressionInvalidTest::test_named_expression_valid_rebinding_iteration_variable: ok")
