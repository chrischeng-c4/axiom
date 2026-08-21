# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_debug_expressions_are_raw_strings"
# subject = "cpython.test_fstring.TestCase.test_debug_expressions_are_raw_strings"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_debug_expressions_are_raw_strings
"""Auto-ported test: TestCase::test_debug_expressions_are_raw_strings (CPython 3.12 oracle)."""


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

assert f"""b"\\N{{OX}}"={b'\\N{OX}'!r}""" == 'b"\\N{OX}"=b\'\\\\N{OX}\''

assert f"""r"\\xff"={'\\xff'!r}""" == 'r"\\xff"=\'\\\\xff\''

assert f"""r"\\n"={'\\n'!r}""" == 'r"\\n"=\'\\\\n\''

assert f"""'\\''={"'"!r}""" == '\'\\\'\'="\'"'

assert f"'\\xc5'={'Å'!r}" == "'\\xc5'='Å'"
print("TestCase::test_debug_expressions_are_raw_strings: ok")
