# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_global"
# subject = "cpython.test_fstring.TestCase.test_global"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_global
"""Auto-ported test: TestCase::test_global (CPython 3.12 oracle)."""


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

assert f'g:{a_global}' == 'g:global variable'

assert f'g:{a_global!r}' == "g:'global variable'"
a_local = 'local variable'

assert f'g:{a_global} l:{a_local}' == 'g:global variable l:local variable'

assert f'g:{a_global!r}' == "g:'global variable'"

assert f'g:{a_global} l:{a_local!r}' == "g:global variable l:'local variable'"

assert "module 'unittest' from" in f'{unittest}'
print("TestCase::test_global: ok")
