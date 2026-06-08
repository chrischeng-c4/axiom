# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_displays"
# subject = "cpython.test_ast.EndPositionTests.test_displays"
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
s1 = '[{}, {1, }, {1, 2,} ]'
s2 = '{a: b, f (): g () ,}'
c1 = _parse_value(s1)
c2 = _parse_value(s2)
_check_content(s1, c1.elts[0], '{}')
_check_content(s1, c1.elts[1], '{1, }')
_check_content(s1, c1.elts[2], '{1, 2,}')
_check_content(s2, c2.keys[1], 'f ()')
_check_content(s2, c2.values[1], 'g ()')

print("EndPositionTests::test_displays: ok")
