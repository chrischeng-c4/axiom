# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "constant_tests__test_get_docstring"
# subject = "cpython.test_ast.ConstantTests.test_get_docstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast


tree = ast.parse("'docstring'\nx = 1")
assert ast.get_docstring(tree) == 'docstring'

print("ConstantTests::test_get_docstring: ok")
