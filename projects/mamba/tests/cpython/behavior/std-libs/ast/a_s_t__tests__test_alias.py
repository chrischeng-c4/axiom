# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_alias"
# subject = "cpython.test_ast.AST_Tests.test_alias"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

im = ast.parse("from bar import y").body[0]
assert len(im.names) == 1
alias = im.names[0]
assert alias.name == "y"
assert alias.asname is None
assert alias.lineno == 1
assert alias.end_lineno == 1
assert alias.col_offset == 16
assert alias.end_col_offset == 17

im = ast.parse("from bar import *").body[0]
alias = im.names[0]
assert alias.name == "*"
assert alias.asname is None
assert alias.lineno == 1
assert alias.end_lineno == 1
assert alias.col_offset == 16
assert alias.end_col_offset == 17

im = ast.parse("from bar import y as z").body[0]
alias = im.names[0]
assert alias.name == "y"
assert alias.asname == "z"
assert alias.lineno == 1
assert alias.end_lineno == 1
assert alias.col_offset == 16
assert alias.end_col_offset == 22

im = ast.parse("import bar as foo").body[0]
alias = im.names[0]
assert alias.name == "bar"
assert alias.asname == "foo"
assert alias.lineno == 1
assert alias.end_lineno == 1
assert alias.col_offset == 7
assert alias.end_col_offset == 17

print("AST_Tests::test_alias: ok")
