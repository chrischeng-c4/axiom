# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_slice"
# subject = "cpython.test_ast.AST_Tests.test_slice"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

slc = ast.parse("x[::]").body[0].value.slice
assert slc.upper is None
assert slc.lower is None
assert slc.step is None

print("AST_Tests::test_slice: ok")
