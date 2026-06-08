# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_ast_compile_time_concat"
# subject = "cpython.test_fstring.TestCase.test_ast_compile_time_concat"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_ast_compile_time_concat
"""Auto-ported test: TestCase::test_ast_compile_time_concat (CPython 3.12 oracle)."""


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
x = ['']
expr = "x[0] = 'foo' f'{3}'"
t = ast.parse(expr)
c = compile(t, '', 'exec')
exec(c)

assert x[0] == 'foo3'
print("TestCase::test_ast_compile_time_concat: ok")
