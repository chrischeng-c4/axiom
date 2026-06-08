# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_ast"
# subject = "cpython.test_fstring.TestCase.test_ast"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_ast
"""Auto-ported test: TestCase::test_ast (CPython 3.12 oracle)."""


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
class X:

    def __init__(self):
        self.called = False

    def __call__(self):
        self.called = True
        return 4
x = X()
expr = "\na = 10\nf'{a * x()}'"
t = ast.parse(expr)
c = compile(t, '', 'exec')

assert not x.called
exec(c)

assert x.called
print("TestCase::test_ast: ok")
