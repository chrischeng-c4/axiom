# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_constant_as_name"
# subject = "cpython.test_ast.AST_Tests.test_constant_as_name"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

for constant in ("True", "False", "None"):
    expr = ast.Expression(ast.Name(constant, ast.Load()))
    ast.fix_missing_locations(expr)
    try:
        compile(expr, "<test>", "eval")
    except ValueError as exc:
        expected = f"identifier field can't represent '{constant}' constant"
        if expected not in str(exc):
            raise AssertionError((constant, str(exc), expected))
    else:
        raise AssertionError(f"expected ValueError for {constant}")

print("AST_Tests::test_constant_as_name: ok")
