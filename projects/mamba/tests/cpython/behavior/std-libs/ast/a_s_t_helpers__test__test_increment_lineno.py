# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_increment_lineno"
# subject = "cpython.test_ast.ASTHelpers_Test.test_increment_lineno"
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
src = ast.parse('1 + 1', mode='eval')
assert ast.increment_lineno(src, n=3) == src
assert ast.dump(src, include_attributes=True) == 'Expression(body=BinOp(left=Constant(value=1, lineno=4, col_offset=0, end_lineno=4, end_col_offset=1), op=Add(), right=Constant(value=1, lineno=4, col_offset=4, end_lineno=4, end_col_offset=5), lineno=4, col_offset=0, end_lineno=4, end_col_offset=5))'
src = ast.parse('1 + 1', mode='eval')
assert ast.increment_lineno(src.body, n=3) == src.body
assert ast.dump(src, include_attributes=True) == 'Expression(body=BinOp(left=Constant(value=1, lineno=4, col_offset=0, end_lineno=4, end_col_offset=1), op=Add(), right=Constant(value=1, lineno=4, col_offset=4, end_lineno=4, end_col_offset=5), lineno=4, col_offset=0, end_lineno=4, end_col_offset=5))'
src = ast.Call(func=ast.Name('test', ast.Load()), args=[], keywords=[], lineno=1)
assert ast.increment_lineno(src).lineno == 2
assert ast.increment_lineno(src).end_lineno is None

print("ASTHelpers_Test::test_increment_lineno: ok")
