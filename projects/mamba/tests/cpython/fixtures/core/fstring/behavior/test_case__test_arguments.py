# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_arguments"
# subject = "cpython.test_fstring.TestCase.test_arguments"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_arguments
"""Auto-ported test: TestCase::test_arguments (CPython 3.12 oracle)."""


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
y = 2

def f(x, width):
    return f'x={x * y:{width}}'

assert f('foo', 10) == 'x=foofoo    '
x = 'bar'

assert f(10, 10) == 'x=        20'
print("TestCase::test_arguments: ok")
