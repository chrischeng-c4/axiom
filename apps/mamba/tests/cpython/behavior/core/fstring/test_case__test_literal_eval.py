# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_literal_eval"
# subject = "cpython.test_fstring.TestCase.test_literal_eval"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_literal_eval
"""Auto-ported test: TestCase::test_literal_eval (CPython 3.12 oracle)."""


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
try:
    ast.literal_eval("f'x'")
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('malformed node or string', str(_aR_e))
print("TestCase::test_literal_eval: ok")
