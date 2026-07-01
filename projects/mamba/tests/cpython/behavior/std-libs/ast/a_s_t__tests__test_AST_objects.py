# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_AST_objects"
# subject = "cpython.test_ast.AST_Tests.test_AST_objects"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

x = ast.AST()
assert x._fields == ()
x.foobar = 42
assert x.foobar == 42
assert x.__dict__["foobar"] == 42

try:
    x.vararg
except AttributeError:
    pass
else:
    raise AssertionError("ast.AST missing attribute did not raise AttributeError")

try:
    ast.AST(2)
except TypeError:
    pass
else:
    raise AssertionError("ast.AST positional argument did not raise TypeError")

print("AST_Tests::test_AST_objects: ok")
