# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_mangled_names_uc354eaf"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_mangled_names"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
class Foo:

    def f(self_):
        global __x1
        __x1 = 0
        [(_Foo__x1 := 1) for a in [2]]
        assert __x1 == 1
        [(__x1 := 2) for a in [3]]
        assert __x1 == 2
Foo().f()
assert _Foo__x1 == 2

print("NamedExpressionScopeTest::test_named_expression_scope_mangled_names: ok")
