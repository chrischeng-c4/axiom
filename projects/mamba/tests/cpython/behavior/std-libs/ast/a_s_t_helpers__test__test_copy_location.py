# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_copy_location"
# subject = "cpython.test_ast.ASTHelpers_Test.test_copy_location"
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
src.body.right = ast.copy_location(ast.Constant(2), src.body.right)
assert ast.dump(src, include_attributes=True) == 'Expression(body=BinOp(left=Constant(value=1, lineno=1, col_offset=0, end_lineno=1, end_col_offset=1), op=Add(), right=Constant(value=2, lineno=1, col_offset=4, end_lineno=1, end_col_offset=5), lineno=1, col_offset=0, end_lineno=1, end_col_offset=5))'
src = ast.Call(col_offset=1, lineno=1, end_lineno=1, end_col_offset=1)
new = ast.copy_location(src, ast.Call(col_offset=None, lineno=None))
assert new.end_lineno is None
assert new.end_col_offset is None
assert new.lineno == 1
assert new.col_offset == 1

print("ASTHelpers_Test::test_copy_location: ok")
