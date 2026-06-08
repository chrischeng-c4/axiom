# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_missing_format_spec"
# subject = "cpython.test_fstring.TestCase.test_missing_format_spec"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_missing_format_spec
"""Auto-ported test: TestCase::test_missing_format_spec (CPython 3.12 oracle)."""


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
class O:

    def __format__(self, spec):
        if not spec:
            return '*'
        return spec

assert f'{O():x}' == 'x'

assert f'{O()}' == '*'

assert f'{O():}' == '*'

assert f'{3:}' == '3'

assert f'{3!s:}' == '3'
print("TestCase::test_missing_format_spec: ok")
