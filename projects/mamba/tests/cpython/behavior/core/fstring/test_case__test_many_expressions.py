# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_many_expressions"
# subject = "cpython.test_fstring.TestCase.test_many_expressions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_many_expressions
"""Auto-ported test: TestCase::test_many_expressions (CPython 3.12 oracle)."""


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
def build_fstr(n, extra=''):
    return "f'" + '{x} ' * n + extra + "'"
x = 'X'
width = 1
for i in range(250, 260):

    assert eval(build_fstr(i)) == (x + ' ') * i

assert eval(build_fstr(255) * 256) == (x + ' ') * (255 * 256)
s = build_fstr(253, '{x:{width}} ')

assert eval(s) == (x + ' ') * 254
s = "f'{1}' 'x' 'y'" * 1024

assert eval(s) == '1xy' * 1024
print("TestCase::test_many_expressions: ok")
