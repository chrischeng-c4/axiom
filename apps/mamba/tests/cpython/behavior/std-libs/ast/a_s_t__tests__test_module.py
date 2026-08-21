# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_module"
# subject = "cpython.test_ast.AST_Tests.test_module"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

body = [ast.Constant(42)]
x = ast.Module(body, [])
assert x.body == body

print("AST_Tests::test_module: ok")
