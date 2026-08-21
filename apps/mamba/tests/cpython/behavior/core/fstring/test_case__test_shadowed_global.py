# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_shadowed_global"
# subject = "cpython.test_fstring.TestCase.test_shadowed_global"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_shadowed_global
"""Auto-ported test: TestCase::test_shadowed_global (CPython 3.12 oracle)."""


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
a_global = 'really a local'

assert f'g:{a_global}' == 'g:really a local'

assert f'g:{a_global!r}' == "g:'really a local'"
a_local = 'local variable'

assert f'g:{a_global} l:{a_local}' == 'g:really a local l:local variable'

assert f'g:{a_global!r}' == "g:'really a local'"

assert f'g:{a_global} l:{a_local!r}' == "g:really a local l:'local variable'"
print("TestCase::test_shadowed_global: ok")
