# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_fix_missing_locations"
# subject = "cpython.test_ast.ASTHelpers_Test.test_fix_missing_locations"
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
src = ast.parse('write("spam")')
src.body.append(ast.Expr(ast.Call(ast.Name('spam', ast.Load()), [ast.Constant('eggs')], [])))
assert src == ast.fix_missing_locations(src)
maxDiff = None
assert ast.dump(src, include_attributes=True) == "Module(body=[Expr(value=Call(func=Name(id='write', ctx=Load(), lineno=1, col_offset=0, end_lineno=1, end_col_offset=5), args=[Constant(value='spam', lineno=1, col_offset=6, end_lineno=1, end_col_offset=12)], keywords=[], lineno=1, col_offset=0, end_lineno=1, end_col_offset=13), lineno=1, col_offset=0, end_lineno=1, end_col_offset=13), Expr(value=Call(func=Name(id='spam', ctx=Load(), lineno=1, col_offset=0, end_lineno=1, end_col_offset=0), args=[Constant(value='eggs', lineno=1, col_offset=0, end_lineno=1, end_col_offset=0)], keywords=[], lineno=1, col_offset=0, end_lineno=1, end_col_offset=0), lineno=1, col_offset=0, end_lineno=1, end_col_offset=0)], type_ignores=[])"

print("ASTHelpers_Test::test_fix_missing_locations: ok")
