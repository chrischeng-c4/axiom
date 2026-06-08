# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_ast_line_numbers"
# subject = "cpython.test_fstring.TestCase.test_ast_line_numbers"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_ast_line_numbers
"""Auto-ported test: TestCase::test_ast_line_numbers (CPython 3.12 oracle)."""


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
expr = "\na = 10\nf'{a * x()}'"
t = ast.parse(expr)

assert type(t) == ast.Module

assert len(t.body) == 2

assert type(t.body[0]) == ast.Assign

assert t.body[0].lineno == 2

assert type(t.body[1]) == ast.Expr

assert type(t.body[1].value) == ast.JoinedStr

assert len(t.body[1].value.values) == 1

assert type(t.body[1].value.values[0]) == ast.FormattedValue

assert t.body[1].lineno == 3

assert t.body[1].value.lineno == 3

assert t.body[1].value.values[0].lineno == 3
binop = t.body[1].value.values[0].value

assert type(binop) == ast.BinOp

assert type(binop.left) == ast.Name

assert type(binop.op) == ast.Mult

assert type(binop.right) == ast.Call

assert binop.lineno == 3

assert binop.left.lineno == 3

assert binop.right.lineno == 3

assert binop.col_offset == 3

assert binop.left.col_offset == 3

assert binop.right.col_offset == 7
print("TestCase::test_ast_line_numbers: ok")
