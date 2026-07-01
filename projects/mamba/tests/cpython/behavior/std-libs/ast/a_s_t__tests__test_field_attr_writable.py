# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_field_attr_writable"
# subject = "cpython.test_ast.AST_Tests.test_field_attr_writable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

x = ast.Constant()
x._fields = 666
assert x._fields == 666

print("AST_Tests::test_field_attr_writable: ok")
