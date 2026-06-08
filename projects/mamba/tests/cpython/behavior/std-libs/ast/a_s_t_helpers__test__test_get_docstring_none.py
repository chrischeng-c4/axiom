# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_get_docstring_none"
# subject = "cpython.test_ast.ASTHelpers_Test.test_get_docstring_none"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
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
maxDiff = None
assert ast.get_docstring(ast.parse('')) is None
node = ast.parse('x = "not docstring"')
assert ast.get_docstring(node) is None
node = ast.parse('def foo():\n  pass')
assert ast.get_docstring(node) is None
node = ast.parse('class foo:\n  pass')
assert ast.get_docstring(node.body[0]) is None
node = ast.parse('class foo:\n  x = "not docstring"')
assert ast.get_docstring(node.body[0]) is None
node = ast.parse('class foo:\n  def bar(self): pass')
assert ast.get_docstring(node.body[0]) is None
node = ast.parse('def foo():\n  pass')
assert ast.get_docstring(node.body[0]) is None
node = ast.parse('def foo():\n  x = "not docstring"')
assert ast.get_docstring(node.body[0]) is None
node = ast.parse('async def foo():\n  pass')
assert ast.get_docstring(node.body[0]) is None
node = ast.parse('async def foo():\n  x = "not docstring"')
assert ast.get_docstring(node.body[0]) is None
node = ast.parse('async def foo():\n  42')
assert ast.get_docstring(node.body[0]) is None

print("ASTHelpers_Test::test_get_docstring_none: ok")
