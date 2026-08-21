# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_not_equal"
# subject = "cpython.test_fstring.TestCase.test_not_equal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_not_equal
"""Auto-ported test: TestCase::test_not_equal (CPython 3.12 oracle)."""


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

assert f'{3 != 4}' == 'True'

assert f'{3 != 4:}' == 'True'

assert f'{3 != 4!s}' == 'True'

assert f'{3 != 4!s:.3}' == 'Tru'
print("TestCase::test_not_equal: ok")
