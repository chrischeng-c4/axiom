# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_nodeclasses"
# subject = "cpython.test_ast.AST_Tests.test_nodeclasses"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast


def assert_type_error(*args, **kwargs):
    try:
        ast.BinOp(*args, **kwargs)
    except TypeError:
        return
    raise AssertionError("expected TypeError from ast.BinOp")


x = ast.BinOp()
assert x._fields == ("left", "op", "right")

x.foobarbaz = 5
assert x.foobarbaz == 5

n1 = ast.Constant(1)
n3 = ast.Constant(3)
addop = ast.Add()
x = ast.BinOp(n1, addop, n3)
assert x.left is n1
assert x.op is addop
assert x.right is n3

x = ast.BinOp(1, 2, 3)
assert x.left == 1
assert x.op == 2
assert x.right == 3

x = ast.BinOp(1, 2, 3, lineno=0)
assert x.left == 1
assert x.op == 2
assert x.right == 3
assert x.lineno == 0

assert_type_error(1, 2, 3, 4)
assert_type_error(1, 2, 3, 4, lineno=0)

x = ast.BinOp(left=1, op=2, right=3, lineno=0)
assert x.left == 1
assert x.op == 2
assert x.right == 3
assert x.lineno == 0

x = ast.BinOp(1, 2, 3, foobarbaz=42)
assert x.foobarbaz == 42

print("AST_Tests::test_nodeclasses: ok")
