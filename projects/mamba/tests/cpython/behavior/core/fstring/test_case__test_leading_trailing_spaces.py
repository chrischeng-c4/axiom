# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_leading_trailing_spaces"
# subject = "cpython.test_fstring.TestCase.test_leading_trailing_spaces"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_leading_trailing_spaces
"""Auto-ported test: TestCase::test_leading_trailing_spaces (CPython 3.12 oracle)."""


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

assert f'{3}' == '3'

assert f'{3}' == '3'

assert f'{3}' == '3'

assert f'{3}' == '3'

assert f'expr={ {x: y for x, y in [(1, 2)]}}' == 'expr={1: 2}'

assert f'expr={ {x: y for x, y in [(1, 2)]}}' == 'expr={1: 2}'
print("TestCase::test_leading_trailing_spaces: ok")
