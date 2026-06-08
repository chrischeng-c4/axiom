# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_suites"
# subject = "cpython.test_ast.EndPositionTests.test_suites"
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
s = dedent('\n            while True:\n                pass\n\n            if one():\n                x = None\n            elif other():\n                y = None\n            else:\n                z = None\n\n            for x, y in stuff:\n                assert True\n\n            try:\n                raise RuntimeError\n            except TypeError as e:\n                pass\n\n            pass\n        ').strip()
mod = ast.parse(s)
while_loop = mod.body[0]
if_stmt = mod.body[1]
for_loop = mod.body[2]
try_stmt = mod.body[3]
pass_stmt = mod.body[4]
_check_end_pos(while_loop, 2, 8)
_check_end_pos(if_stmt, 9, 12)
_check_end_pos(for_loop, 12, 15)
_check_end_pos(try_stmt, 17, 8)
_check_end_pos(pass_stmt, 19, 4)
_check_content(s, while_loop.test, 'True')
_check_content(s, if_stmt.body[0], 'x = None')
_check_content(s, if_stmt.orelse[0].test, 'other()')
_check_content(s, for_loop.target, 'x, y')
_check_content(s, try_stmt.body[0], 'raise RuntimeError')
_check_content(s, try_stmt.handlers[0].type, 'TypeError')

print("EndPositionTests::test_suites: ok")
