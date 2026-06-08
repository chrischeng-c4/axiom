# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_slices"
# subject = "cpython.test_ast.EndPositionTests.test_slices"
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
s1 = 'f()[1, 2] [0]'
s2 = 'x[ a.b: c.d]'
sm = dedent('\n            x[ a.b: f () ,\n               g () : c.d\n              ]\n        ').strip()
i1, i2, im = map(_parse_value, (s1, s2, sm))
_check_content(s1, i1.value, 'f()[1, 2]')
_check_content(s1, i1.value.slice, '1, 2')
_check_content(s2, i2.slice.lower, 'a.b')
_check_content(s2, i2.slice.upper, 'c.d')
_check_content(sm, im.slice.elts[0].upper, 'f ()')
_check_content(sm, im.slice.elts[1].lower, 'g ()')
_check_end_pos(im, 3, 3)

print("EndPositionTests::test_slices: ok")
