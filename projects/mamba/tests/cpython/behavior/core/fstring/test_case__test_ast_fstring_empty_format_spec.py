# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_ast_fstring_empty_format_spec"
# subject = "cpython.test_fstring.TestCase.test_ast_fstring_empty_format_spec"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_ast_fstring_empty_format_spec
"""Auto-ported test: TestCase::test_ast_fstring_empty_format_spec (CPython 3.12 oracle)."""


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
expr = "f'{expr:}'"
mod = ast.parse(expr)

assert type(mod) == ast.Module

assert len(mod.body) == 1
fstring = mod.body[0].value

assert type(fstring) == ast.JoinedStr

assert len(fstring.values) == 1
fv = fstring.values[0]

assert type(fv) == ast.FormattedValue
format_spec = fv.format_spec

assert type(format_spec) == ast.JoinedStr

assert len(format_spec.values) == 0
print("TestCase::test_ast_fstring_empty_format_spec: ok")
