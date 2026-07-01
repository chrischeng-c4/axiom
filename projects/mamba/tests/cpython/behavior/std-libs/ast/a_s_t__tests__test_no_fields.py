# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_no_fields"
# subject = "cpython.test_ast.AST_Tests.test_no_fields"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

x = ast.Sub()
assert x._fields == ()

print("AST_Tests::test_no_fields: ok")
