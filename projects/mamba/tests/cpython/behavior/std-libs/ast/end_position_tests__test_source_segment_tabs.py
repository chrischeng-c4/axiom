# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_source_segment_tabs"
# subject = "cpython.test_ast.EndPositionTests.test_source_segment_tabs"
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
s = dedent('\n            class C:\n              \t\x0c  def fun(self) -> None:\n              \t\x0c      pass\n        ').strip()
s_method = '  \t\x0c  def fun(self) -> None:\n  \t\x0c      pass'
cdef = ast.parse(s).body[0]
assert ast.get_source_segment(s, cdef.body[0], padded=True) == s_method

print("EndPositionTests::test_source_segment_tabs: ok")
