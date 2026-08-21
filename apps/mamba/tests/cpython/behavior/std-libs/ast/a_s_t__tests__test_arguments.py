# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_arguments"
# subject = "cpython.test_ast.AST_Tests.test_arguments"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

x = ast.arguments()
expected_fields = (
    "posonlyargs",
    "args",
    "vararg",
    "kwonlyargs",
    "kw_defaults",
    "kwarg",
    "defaults",
)
if x._fields != expected_fields:
    raise AssertionError((x._fields, expected_fields))

try:
    x.args
except AttributeError:
    pass
else:
    raise AssertionError("ast.arguments().args should be missing")

if x.vararg is not None:
    raise AssertionError(x.vararg)

x = ast.arguments(*range(1, 8))
if x.args != 2:
    raise AssertionError(x.args)
if x.vararg != 3:
    raise AssertionError(x.vararg)

print("AST_Tests::test_arguments: ok")
