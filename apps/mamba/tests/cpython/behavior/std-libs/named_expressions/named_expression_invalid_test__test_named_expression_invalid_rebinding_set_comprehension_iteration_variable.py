# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_invalid_test__test_named_expression_invalid_rebinding_set_comprehension_iteration_variable"
# subject = "cpython.test_named_expressions.NamedExpressionInvalidTest.test_named_expression_invalid_rebinding_set_comprehension_iteration_variable"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionInvalidTest::test_named_expression_invalid_rebinding_set_comprehension_iteration_variable
"""Auto-ported test: NamedExpressionInvalidTest::test_named_expression_invalid_rebinding_set_comprehension_iteration_variable (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
cases = [('Local reuse', 'i', '{i := 0 for i in range(5)}'), ('Nested reuse', 'j', '{{(j := 0) for i in range(5)} for j in range(5)}'), ('Reuse inner loop target', 'j', '{(j := 0) for i in range(5) for j in range(5)}'), ('Unpacking reuse', 'i', '{i := 0 for i, j in {(0, 1)}}'), ('Reuse in loop condition', 'i', '{i+1 for i in range(5) if (i := 0)}'), ('Unreachable reuse', 'i', '{False or (i:=0) for i in range(5)}'), ('Unreachable nested reuse', 'i', '{(i, j) for i in range(5) for j in range(5) if True or (i:=10)}'), ('Complex expression: a', 'a', '{(a := 1) for a, (*b, c[d+e::f(g)], h.i) in j}'), ('Complex expression: b', 'b', '{(b := 1) for a, (*b, c[d+e::f(g)], h.i) in j}')]
for case, target, code in cases:
    msg = f"assignment expression cannot rebind comprehension iteration variable '{target}'"
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
print("NamedExpressionInvalidTest::test_named_expression_invalid_rebinding_set_comprehension_iteration_variable: ok")
