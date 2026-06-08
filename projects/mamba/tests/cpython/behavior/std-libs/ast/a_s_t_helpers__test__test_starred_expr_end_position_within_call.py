# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t_helpers__test__test_starred_expr_end_position_within_call"
# subject = "cpython.test_ast.ASTHelpers_Test.test_starred_expr_end_position_within_call"
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
node = ast.parse('f(*[0, 1])')
starred_expr = node.body[0].value.args[0]
assert starred_expr.end_lineno == 1
assert starred_expr.end_col_offset == 9

print("ASTHelpers_Test::test_starred_expr_end_position_within_call: ok")
