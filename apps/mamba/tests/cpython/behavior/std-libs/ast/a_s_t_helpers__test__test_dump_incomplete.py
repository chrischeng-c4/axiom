# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_dump_incomplete"
# subject = "cpython.test_ast.ASTHelpers_Test.test_dump_incomplete"
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
node = ast.Raise(lineno=3, col_offset=4)
assert ast.dump(node) == 'Raise()'
assert ast.dump(node, include_attributes=True) == 'Raise(lineno=3, col_offset=4)'
node = ast.Raise(exc=ast.Name(id='e', ctx=ast.Load()), lineno=3, col_offset=4)
assert ast.dump(node) == "Raise(exc=Name(id='e', ctx=Load()))"
assert ast.dump(node, annotate_fields=False) == "Raise(Name('e', Load()))"
assert ast.dump(node, include_attributes=True) == "Raise(exc=Name(id='e', ctx=Load()), lineno=3, col_offset=4)"
assert ast.dump(node, annotate_fields=False, include_attributes=True) == "Raise(Name('e', Load()), lineno=3, col_offset=4)"
node = ast.Raise(cause=ast.Name(id='e', ctx=ast.Load()))
assert ast.dump(node) == "Raise(cause=Name(id='e', ctx=Load()))"
assert ast.dump(node, annotate_fields=False) == "Raise(cause=Name('e', Load()))"

print("ASTHelpers_Test::test_dump_incomplete: ok")
