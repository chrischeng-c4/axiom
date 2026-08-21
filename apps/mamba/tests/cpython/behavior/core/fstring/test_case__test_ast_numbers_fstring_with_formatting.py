# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_ast_numbers_fstring_with_formatting"
# subject = "cpython.test_fstring.TestCase.test_ast_numbers_fstring_with_formatting"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_ast_numbers_fstring_with_formatting
"""Auto-ported test: TestCase::test_ast_numbers_fstring_with_formatting (CPython 3.12 oracle)."""


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
t = ast.parse('f"Here is that pesky {xxx:.3f} again"')

assert len(t.body) == 1

assert t.body[0].lineno == 1

assert type(t.body[0]) == ast.Expr

assert type(t.body[0].value) == ast.JoinedStr

assert len(t.body[0].value.values) == 3

assert type(t.body[0].value.values[0]) == ast.Constant

assert type(t.body[0].value.values[1]) == ast.FormattedValue

assert type(t.body[0].value.values[2]) == ast.Constant
_, expr, _ = t.body[0].value.values
name = expr.value

assert type(name) == ast.Name

assert name.lineno == 1

assert name.end_lineno == 1

assert name.col_offset == 22

assert name.end_col_offset == 25
print("TestCase::test_ast_numbers_fstring_with_formatting: ok")
