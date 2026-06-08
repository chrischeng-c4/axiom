# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_nonlocal_scope_uc0e372a"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_nonlocal_scope"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
sentinel = object()

def f():
    nonlocal_var = None

    def g():
        nonlocal nonlocal_var
        [(nonlocal_var := sentinel) for _ in range(1)]
    g()
    assert nonlocal_var == sentinel
f()

print("NamedExpressionScopeTest::test_named_expression_nonlocal_scope: ok")
