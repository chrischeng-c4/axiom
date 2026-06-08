# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_global_scope_uc5521b4"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_global_scope"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
sentinel = object()
global GLOBAL_VAR

def f():
    global GLOBAL_VAR
    [(GLOBAL_VAR := sentinel) for _ in range(1)]
    assert GLOBAL_VAR == sentinel
try:
    f()
    assert GLOBAL_VAR == sentinel
finally:
    GLOBAL_VAR = None

print("NamedExpressionScopeTest::test_named_expression_global_scope: ok")
