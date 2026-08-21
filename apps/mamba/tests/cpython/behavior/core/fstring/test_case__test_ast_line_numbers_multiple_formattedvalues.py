# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_ast_line_numbers_multiple_formattedvalues"
# subject = "cpython.test_fstring.TestCase.test_ast_line_numbers_multiple_formattedvalues"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_ast_line_numbers_multiple_formattedvalues
"""Auto-ported test: TestCase::test_ast_line_numbers_multiple_formattedvalues (CPython 3.12 oracle)."""


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
expr = "\nf'no formatted values'\nf'eggs {a * x()} spam {b + y()}'"
t = ast.parse(expr)

assert type(t) == ast.Module

assert len(t.body) == 2

assert type(t.body[0]) == ast.Expr

assert type(t.body[0].value) == ast.JoinedStr

assert t.body[0].lineno == 2

assert type(t.body[1]) == ast.Expr

assert type(t.body[1].value) == ast.JoinedStr

assert len(t.body[1].value.values) == 4

assert type(t.body[1].value.values[0]) == ast.Constant

assert type(t.body[1].value.values[0].value) == str

assert type(t.body[1].value.values[1]) == ast.FormattedValue

assert type(t.body[1].value.values[2]) == ast.Constant

assert type(t.body[1].value.values[2].value) == str

assert type(t.body[1].value.values[3]) == ast.FormattedValue

assert t.body[1].lineno == 3

assert t.body[1].value.lineno == 3

assert t.body[1].value.values[0].lineno == 3

assert t.body[1].value.values[1].lineno == 3

assert t.body[1].value.values[2].lineno == 3

assert t.body[1].value.values[3].lineno == 3
binop1 = t.body[1].value.values[1].value

assert type(binop1) == ast.BinOp

assert type(binop1.left) == ast.Name

assert type(binop1.op) == ast.Mult

assert type(binop1.right) == ast.Call

assert binop1.lineno == 3

assert binop1.left.lineno == 3

assert binop1.right.lineno == 3

assert binop1.col_offset == 8

assert binop1.left.col_offset == 8

assert binop1.right.col_offset == 12
binop2 = t.body[1].value.values[3].value

assert type(binop2) == ast.BinOp

assert type(binop2.left) == ast.Name

assert type(binop2.op) == ast.Add

assert type(binop2.right) == ast.Call

assert binop2.lineno == 3

assert binop2.left.lineno == 3

assert binop2.right.lineno == 3

assert binop2.col_offset == 23

assert binop2.left.col_offset == 23

assert binop2.right.col_offset == 27
print("TestCase::test_ast_line_numbers_multiple_formattedvalues: ok")
