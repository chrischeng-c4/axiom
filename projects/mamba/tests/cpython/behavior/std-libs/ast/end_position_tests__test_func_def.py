# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "end_position_tests__test_func_def"
# subject = "cpython.test_ast.EndPositionTests.test_func_def"
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
s = dedent('\n            def func(x: int,\n                     *args: str,\n                     z: float = 0,\n                     **kwargs: Any) -> bool:\n                return True\n            ').strip()
fdef = ast.parse(s).body[0]
_check_end_pos(fdef, 5, 15)
_check_content(s, fdef.body[0], 'return True')
_check_content(s, fdef.args.args[0], 'x: int')
_check_content(s, fdef.args.args[0].annotation, 'int')
_check_content(s, fdef.args.kwarg, 'kwargs: Any')
_check_content(s, fdef.args.kwarg.annotation, 'Any')

print("EndPositionTests::test_func_def: ok")
