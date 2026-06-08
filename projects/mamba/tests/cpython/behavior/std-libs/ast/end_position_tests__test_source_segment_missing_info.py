# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_source_segment_missing_info"
# subject = "cpython.test_ast.EndPositionTests.test_source_segment_missing_info"
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
s = 'v = 1\r\nw = 1\nx = 1\n\ry = 1\r\n'
v, w, x, y = ast.parse(s).body
del v.lineno
del w.end_lineno
del x.col_offset
del y.end_col_offset
assert ast.get_source_segment(s, v) is None
assert ast.get_source_segment(s, w) is None
assert ast.get_source_segment(s, x) is None
assert ast.get_source_segment(s, y) is None

print("EndPositionTests::test_source_segment_missing_info: ok")
