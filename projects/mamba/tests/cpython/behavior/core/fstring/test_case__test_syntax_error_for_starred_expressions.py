# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_syntax_error_for_starred_expressions"
# subject = "cpython.test_fstring.TestCase.test_syntax_error_for_starred_expressions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_syntax_error_for_starred_expressions
"""Auto-ported test: TestCase::test_syntax_error_for_starred_expressions (CPython 3.12 oracle)."""


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
    compile("f'{*a}'", '?', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import re as _re_aR
    assert _re_aR.search("can't use starred expression here", str(_aR_e))
try:
    compile("f'{**a}'", '?', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import re as _re_aR
    assert _re_aR.search("f-string: expecting a valid expression after '{'", str(_aR_e))
print("TestCase::test_syntax_error_for_starred_expressions: ok")
