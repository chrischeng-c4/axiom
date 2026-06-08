# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_redundant_parenthesis"
# subject = "cpython.test_ast.EndPositionTests.test_redundant_parenthesis"
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
s = '( ( ( a + b ) ) )'
v = ast.parse(s).body[0].value
assert type(v).__name__ == 'BinOp'
_check_content(s, v, 'a + b')
s2 = 'await ' + s
v = ast.parse(s2).body[0].value.value
assert type(v).__name__ == 'BinOp'
_check_content(s2, v, 'a + b')

print("EndPositionTests::test_redundant_parenthesis: ok")
