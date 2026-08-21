# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_empty_format_specifier"
# subject = "cpython.test_fstring.TestCase.test_empty_format_specifier"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_empty_format_specifier
"""Auto-ported test: TestCase::test_empty_format_specifier (CPython 3.12 oracle)."""


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
x = 'test'

assert f'{x}' == 'test'

assert f'{x:}' == 'test'

assert f'{x!s:}' == 'test'

assert f'{x!r:}' == "'test'"
print("TestCase::test_empty_format_specifier: ok")
