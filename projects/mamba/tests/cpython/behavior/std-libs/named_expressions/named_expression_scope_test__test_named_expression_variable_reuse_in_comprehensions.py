# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_variable_reuse_in_comprehensions"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_variable_reuse_in_comprehensions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_variable_reuse_in_comprehensions
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_variable_reuse_in_comprehensions (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
rebinding = '[x := i for i in range(3) if (x := i) or not x]'
filter_ref = '[x := i for i in range(3) if x or not x]'
body_ref = '[x for i in range(3) if (x := i) or not x]'
nested_ref = '[j for i in range(3) if x or not x for j in range(3) if (x := i)][:-3]'
cases = [('Rebind global', f'x = 1; result = {rebinding}'), ('Rebind nonlocal', f'result, x = (lambda x=1: ({rebinding}, x))()'), ('Filter global', f'x = 1; result = {filter_ref}'), ('Filter nonlocal', f'result, x = (lambda x=1: ({filter_ref}, x))()'), ('Body global', f'x = 1; result = {body_ref}'), ('Body nonlocal', f'result, x = (lambda x=1: ({body_ref}, x))()'), ('Nested global', f'x = 1; result = {nested_ref}'), ('Nested nonlocal', f'result, x = (lambda x=1: ({nested_ref}, x))()')]
for case, code in cases:
    ns = {}
    exec(code, ns)

    assert ns['x'] == 2

    assert ns['result'] == [0, 1, 2]
print("NamedExpressionScopeTest::test_named_expression_variable_reuse_in_comprehensions: ok")
