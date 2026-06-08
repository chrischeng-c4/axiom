# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_ast_line_numbers_multiline_fstring"
# subject = "cpython.test_fstring.TestCase.test_ast_line_numbers_multiline_fstring"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_ast_line_numbers_multiline_fstring
"""Auto-ported test: TestCase::test_ast_line_numbers_multiline_fstring (CPython 3.12 oracle)."""


import ast
import datetime
import os
import re
import types
import decimal
import unittest
import warnings
from test import support
from test.support.os_helper import temp_cwd
from test.support.script_helper import assert_python_failure, assert_python_ok


a_global = 'global variable'


# --- test body ---
expr = "\na = 10\nf'''\n  {a\n     *\n       x()}\nnon-important content\n'''\n"
t = ast.parse(expr)

assert type(t) == ast.Module

assert len(t.body) == 2

assert type(t.body[0]) == ast.Assign

assert t.body[0].lineno == 2

assert type(t.body[1]) == ast.Expr

assert type(t.body[1].value) == ast.JoinedStr

assert len(t.body[1].value.values) == 3

assert type(t.body[1].value.values[0]) == ast.Constant

assert type(t.body[1].value.values[0].value) == str

assert type(t.body[1].value.values[1]) == ast.FormattedValue

assert type(t.body[1].value.values[2]) == ast.Constant

assert type(t.body[1].value.values[2].value) == str

assert t.body[1].lineno == 3

assert t.body[1].value.lineno == 3

assert t.body[1].value.values[0].lineno == 3

assert t.body[1].value.values[1].lineno == 4

assert t.body[1].value.values[2].lineno == 6

assert t.body[1].col_offset == 0

assert t.body[1].value.col_offset == 0

assert t.body[1].value.values[0].col_offset == 4

assert t.body[1].value.values[1].col_offset == 2

assert t.body[1].value.values[2].col_offset == 11
binop = t.body[1].value.values[1].value

assert type(binop) == ast.BinOp

assert type(binop.left) == ast.Name

assert type(binop.op) == ast.Mult

assert type(binop.right) == ast.Call

assert binop.lineno == 4

assert binop.left.lineno == 4

assert binop.right.lineno == 6

assert binop.col_offset == 3

assert binop.left.col_offset == 3

assert binop.right.col_offset == 7
expr = "\na = f'''\n          {blech}\n    '''\n"
t = ast.parse(expr)

assert type(t) == ast.Module

assert len(t.body) == 1

assert type(t.body[0]) == ast.Assign

assert type(t.body[0].value) == ast.JoinedStr

assert len(t.body[0].value.values) == 3

assert type(t.body[0].value.values[1]) == ast.FormattedValue

assert t.body[0].lineno == 2

assert t.body[0].value.lineno == 2

assert t.body[0].value.values[0].lineno == 2

assert t.body[0].value.values[1].lineno == 3

assert t.body[0].value.values[2].lineno == 3

assert t.body[0].col_offset == 0

assert t.body[0].value.col_offset == 4

assert t.body[0].value.values[0].col_offset == 8

assert t.body[0].value.values[1].col_offset == 10

assert t.body[0].value.values[2].col_offset == 17

assert t.body[0].value.values[1].value.lineno == 3

assert t.body[0].value.values[1].value.end_lineno == 3

assert t.body[0].value.values[1].value.col_offset == 11

assert t.body[0].value.values[1].value.end_col_offset == 16
print("TestCase::test_ast_line_numbers_multiline_fstring: ok")
