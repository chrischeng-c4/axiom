# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_invalid_test__test_named_expression_invalid_rebinding_list_comprehension_inner_loop"
# subject = "cpython.test_named_expressions.NamedExpressionInvalidTest.test_named_expression_invalid_rebinding_list_comprehension_inner_loop"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionInvalidTest::test_named_expression_invalid_rebinding_list_comprehension_inner_loop
"""Auto-ported test: NamedExpressionInvalidTest::test_named_expression_invalid_rebinding_list_comprehension_inner_loop (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
cases = [('Inner reuse', 'j', '[i for i in range(5) if (j := 0) for j in range(5)]'), ('Inner unpacking reuse', 'j', '[i for i in range(5) if (j := 0) for j, k in [(0, 1)]]')]
for case, target, code in cases:
    msg = f"comprehension inner loop cannot rebind assignment expression target '{target}'"
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
print("NamedExpressionInvalidTest::test_named_expression_invalid_rebinding_list_comprehension_inner_loop: ok")
