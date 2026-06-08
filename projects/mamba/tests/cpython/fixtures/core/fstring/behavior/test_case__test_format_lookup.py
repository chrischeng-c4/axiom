# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_format_lookup"
# subject = "cpython.test_fstring.TestCase.test__format__lookup"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test__format__lookup
"""Auto-ported test: TestCase::test__format__lookup (CPython 3.12 oracle)."""


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

    def __format__(self, spec):
        return 'class'
x = X()
y = X()
y.__format__ = types.MethodType(lambda self, spec: 'instance', y)

assert f'{y}' == format(y)

assert f'{y}' == 'class'

assert format(x) == format(y)

assert x.__format__('') == 'class'

assert y.__format__('') == 'instance'

assert type(x).__format__(x, '') == 'class'

assert type(y).__format__(y, '') == 'class'
print("TestCase::test__format__lookup: ok")
