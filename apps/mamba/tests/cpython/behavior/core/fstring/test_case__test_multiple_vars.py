# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_multiple_vars"
# subject = "cpython.test_fstring.TestCase.test_multiple_vars"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_multiple_vars
"""Auto-ported test: TestCase::test_multiple_vars (CPython 3.12 oracle)."""


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
x = 98
y = 'abc'

assert f'{x}{y}' == '98abc'

assert f'X{x}{y}' == 'X98abc'

assert f'{x}X{y}' == '98Xabc'

assert f'{x}{y}X' == '98abcX'

assert f'X{x}Y{y}' == 'X98Yabc'

assert f'X{x}{y}Y' == 'X98abcY'

assert f'{x}X{y}Y' == '98XabcY'

assert f'X{x}Y{y}Z' == 'X98YabcZ'
print("TestCase::test_multiple_vars: ok")
