# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_from_import"
# subject = "cpython.test_ast.AST_Tests.test_from_import"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

im = ast.parse("from . import y").body[0]
assert im.module is None

print("AST_Tests::test_from_import: ok")
