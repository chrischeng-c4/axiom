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
tree = ast.parse("'docstring'\nx = 1")
assert ast.get_docstring(tree) == 'docstring'

print("ConstantTests::test_get_docstring: ok")
