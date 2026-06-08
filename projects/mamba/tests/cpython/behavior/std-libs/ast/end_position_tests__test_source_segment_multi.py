# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_source_segment_multi"
# subject = "cpython.test_ast.EndPositionTests.test_source_segment_multi"
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
s_orig = dedent('\n            x = (\n                a, b,\n            ) + ()\n        ').strip()
s_tuple = dedent('\n            (\n                a, b,\n            )\n        ').strip()
binop = _parse_value(s_orig)
assert ast.get_source_segment(s_orig, binop.left) == s_tuple

print("EndPositionTests::test_source_segment_multi: ok")
