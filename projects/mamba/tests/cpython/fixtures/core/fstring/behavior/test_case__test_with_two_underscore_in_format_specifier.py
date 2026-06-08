# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_with_two_underscore_in_format_specifier"
# subject = "cpython.test_fstring.TestCase.test_with_two_underscore_in_format_specifier"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_with_two_underscore_in_format_specifier
"""Auto-ported test: TestCase::test_with_two_underscore_in_format_specifier (CPython 3.12 oracle)."""


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
error_msg = re.escape("Cannot specify '_' with '_'.")
try:
    f'{1:__}'
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(error_msg, str(_aR_e))
print("TestCase::test_with_two_underscore_in_format_specifier: ok")
