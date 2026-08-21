# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_dump"
# subject = "cpython.test_ast.ASTHelpers_Test.test_dump"
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
node = ast.parse('spam(eggs, "and cheese")')
assert ast.dump(node) == "Module(body=[Expr(value=Call(func=Name(id='spam', ctx=Load()), args=[Name(id='eggs', ctx=Load()), Constant(value='and cheese')], keywords=[]))], type_ignores=[])"
assert ast.dump(node, annotate_fields=False) == "Module([Expr(Call(Name('spam', Load()), [Name('eggs', Load()), Constant('and cheese')], []))], [])"
assert ast.dump(node, include_attributes=True) == "Module(body=[Expr(value=Call(func=Name(id='spam', ctx=Load(), lineno=1, col_offset=0, end_lineno=1, end_col_offset=4), args=[Name(id='eggs', ctx=Load(), lineno=1, col_offset=5, end_lineno=1, end_col_offset=9), Constant(value='and cheese', lineno=1, col_offset=11, end_lineno=1, end_col_offset=23)], keywords=[], lineno=1, col_offset=0, end_lineno=1, end_col_offset=24), lineno=1, col_offset=0, end_lineno=1, end_col_offset=24)], type_ignores=[])"

print("ASTHelpers_Test::test_dump: ok")
