# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_iter_child_nodes"
# subject = "cpython.test_ast.ASTHelpers_Test.test_iter_child_nodes"
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
maxDiff = None
node = ast.parse("spam(23, 42, eggs='leek')", mode='eval')
assert len(list(ast.iter_child_nodes(node.body))) == 4
iterator = ast.iter_child_nodes(node.body)
assert next(iterator).id == 'spam'
assert next(iterator).value == 23
assert next(iterator).value == 42
assert ast.dump(next(iterator)) == "keyword(arg='eggs', value=Constant(value='leek'))"

print("ASTHelpers_Test::test_iter_child_nodes: ok")
