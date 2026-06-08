# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_nonlocal_scope_no_nonlocal_keyword_ucfb3f42"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_nonlocal_scope_no_nonlocal_keyword"
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
        [(nonlocal_var := sentinel) for _ in range(1)]
    g()
    assert nonlocal_var == None
f()

print("NamedExpressionScopeTest::test_named_expression_nonlocal_scope_no_nonlocal_keyword: ok")
