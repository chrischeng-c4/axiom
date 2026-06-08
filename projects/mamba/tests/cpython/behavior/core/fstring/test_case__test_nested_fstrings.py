# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_nested_fstrings"
# subject = "cpython.test_fstring.TestCase.test_nested_fstrings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_nested_fstrings
"""Auto-ported test: TestCase::test_nested_fstrings (CPython 3.12 oracle)."""


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
y = 5

assert f"{f'{0}' * 3}" == '000'

assert f"{f'{y}' * 3}" == '555'
print("TestCase::test_nested_fstrings: ok")
