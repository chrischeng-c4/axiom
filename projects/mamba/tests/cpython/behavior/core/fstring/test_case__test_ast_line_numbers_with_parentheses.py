# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_ast_line_numbers_with_parentheses"
# subject = "cpython.test_fstring.TestCase.test_ast_line_numbers_with_parentheses"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_ast_line_numbers_with_parentheses
"""Auto-ported test: TestCase::test_ast_line_numbers_with_parentheses (CPython 3.12 oracle)."""


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
expr = '\nx = (\n    f" {test(t)}"\n)'
t = ast.parse(expr)

assert type(t) == ast.Module

assert len(t.body) == 1
joinedstr = t.body[0].value

assert type(joinedstr) == ast.JoinedStr

assert joinedstr.lineno == 3

assert joinedstr.end_lineno == 3

assert joinedstr.col_offset == 4

assert joinedstr.end_col_offset == 17
fv = t.body[0].value.values[1]

assert type(fv) == ast.FormattedValue

assert fv.lineno == 3

assert fv.end_lineno == 3

assert fv.col_offset == 7

assert fv.end_col_offset == 16
call = t.body[0].value.values[1].value

assert type(call) == ast.Call

assert call.lineno == 3

assert call.end_lineno == 3

assert call.col_offset == 8

assert call.end_col_offset == 15
expr = '\nx = (\n    u\'wat\',\n    u"wat",\n    b\'wat\',\n    b"wat",\n    f\'wat\',\n    f"wat",\n)\n\ny = (\n    u\'\'\'wat\'\'\',\n    u"""wat""",\n    b\'\'\'wat\'\'\',\n    b"""wat""",\n    f\'\'\'wat\'\'\',\n    f"""wat""",\n)\n        '
t = ast.parse(expr)

assert type(t) == ast.Module

assert len(t.body) == 2
x, y = t.body
offsets = [(elt.col_offset, elt.end_col_offset) for elt in x.value.elts]

assert all((offset == (4, 10) for offset in offsets))
offsets = [(elt.col_offset, elt.end_col_offset) for elt in y.value.elts]

assert all((offset == (4, 14) for offset in offsets))
expr = "\nx = (\n        'PERL_MM_OPT', (\n            f'wat'\n            f'some_string={f(x)} '\n            f'wat'\n        ),\n)\n"
t = ast.parse(expr)

assert type(t) == ast.Module

assert len(t.body) == 1
fstring = t.body[0].value.elts[1]

assert type(fstring) == ast.JoinedStr

assert len(fstring.values) == 3
wat1, middle, wat2 = fstring.values

assert type(wat1) == ast.Constant

assert wat1.lineno == 4

assert wat1.end_lineno == 5

assert wat1.col_offset == 14

assert wat1.end_col_offset == 26
call = middle.value

assert type(call) == ast.Call

assert call.lineno == 5

assert call.end_lineno == 5

assert call.col_offset == 27

assert call.end_col_offset == 31

assert type(wat2) == ast.Constant

assert wat2.lineno == 5

assert wat2.end_lineno == 6

assert wat2.col_offset == 32

assert wat2.end_col_offset == 17

assert fstring.end_col_offset == 18
print("TestCase::test_ast_line_numbers_with_parentheses: ok")
