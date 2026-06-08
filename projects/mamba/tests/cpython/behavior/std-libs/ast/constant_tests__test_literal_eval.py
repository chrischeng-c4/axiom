# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "constant_tests__test_literal_eval"
# subject = "cpython.test_ast.ConstantTests.test_literal_eval"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import ast
import builtins
import dis
import enum
import os
import re
import sys
import textwrap
import types
import warnings
import weakref
from functools import partial
from textwrap import dedent

def compile_constant(value):
    tree = ast.parse('x = 123')
    node = tree.body[0].value
    new_node = ast.Constant(value=value)
    ast.copy_location(new_node, node)
    tree.body[0].value = new_node
    code = compile(tree, '<string>', 'exec')
    ns = {}
    exec(code, ns)
    return ns['x']

def get_load_const(tree):
    co = compile(tree, '<string>', 'exec')
    consts = []
    for instr in dis.get_instructions(co):
        if instr.opname == 'LOAD_CONST' or instr.opname == 'RETURN_CONST':
            consts.append(instr.argval)
    return consts
tree = ast.parse('1 + 2')
binop = tree.body[0].value
new_left = ast.Constant(value=10)
ast.copy_location(new_left, binop.left)
binop.left = new_left
new_right = ast.Constant(value=20j)
ast.copy_location(new_right, binop.right)
binop.right = new_right
assert ast.literal_eval(binop) == 10 + 20j

print("ConstantTests::test_literal_eval: ok")
