# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_gh129093"
# subject = "cpython.test_fstring.TestCase.test_gh129093"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_gh129093
"""Auto-ported test: TestCase::test_gh129093 (CPython 3.12 oracle)."""


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

assert f'1==2={1 == 2!r}' == '1==2=False'

assert f'1 == 2={1 == 2!r}' == '1 == 2=False'

assert f'1!=2={1 != 2!r}' == '1!=2=True'

assert f'1 != 2={1 != 2!r}' == '1 != 2=True'

assert f'(1) != 2={1 != 2!r}' == '(1) != 2=True'

assert f'(1*2) != (3)={1 * 2 != 3!r}' == '(1*2) != (3)=True'

assert f'1 != 2 == 3 != 4={1 != 2 == 3 != 4!r}' == '1 != 2 == 3 != 4=False'

assert f'1 == 2 != 3 == 4={1 == 2 != 3 == 4!r}' == '1 == 2 != 3 == 4=False'

assert f"f'{{1==2=}}'={f'1==2={1 == 2!r}'!r}" == "f'{1==2=}'='1==2=False'"

assert f"f'{{1 == 2=}}'={f'1 == 2={1 == 2!r}'!r}" == "f'{1 == 2=}'='1 == 2=False'"

assert f"f'{{1!=2=}}'={f'1!=2={1 != 2!r}'!r}" == "f'{1!=2=}'='1!=2=True'"

assert f"f'{{1 != 2=}}'={f'1 != 2={1 != 2!r}'!r}" == "f'{1 != 2=}'='1 != 2=True'"
print("TestCase::test_gh129093: ok")
