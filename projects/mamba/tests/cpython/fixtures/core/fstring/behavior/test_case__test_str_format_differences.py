# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_str_format_differences"
# subject = "cpython.test_fstring.TestCase.test_str_format_differences"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_str_format_differences
"""Auto-ported test: TestCase::test_str_format_differences (CPython 3.12 oracle)."""


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
d = {'a': 'string', 0: 'integer'}
a = 0

assert f'{d[0]}' == 'integer'

assert f"{d['a']}" == 'string'

assert f'{d[a]}' == 'integer'

assert '{d[a]}'.format(d=d) == 'string'

assert '{d[0]}'.format(d=d) == 'integer'
print("TestCase::test_str_format_differences: ok")
