# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_fstring_multi_line"
# subject = "cpython.test_ast.EndPositionTests.test_fstring_multi_line"
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

def _check_end_pos(ast_node, end_lineno, end_col_offset):
    assert ast_node.end_lineno == end_lineno
    assert ast_node.end_col_offset == end_col_offset

def _check_content(source, ast_node, content):
    assert ast.get_source_segment(source, ast_node) == content

def _parse_value(s):
    return ast.parse(s).body[0].value
s = dedent('\n            f"""Some multi-line text.\n            {\n            arg_one\n            +\n            arg_two\n            }\n            It goes on..."""\n        ').strip()
fstr = _parse_value(s)
binop = fstr.values[1].value
_check_end_pos(binop, 5, 7)
_check_content(s, binop.left, 'arg_one')
_check_content(s, binop.right, 'arg_two')

print("EndPositionTests::test_fstring_multi_line: ok")
