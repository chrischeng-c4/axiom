# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_syntax_warning_infinite_recursion_in_file"
# subject = "cpython.test_fstring.TestCase.test_syntax_warning_infinite_recursion_in_file"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_syntax_warning_infinite_recursion_in_file
"""Auto-ported test: TestCase::test_syntax_warning_infinite_recursion_in_file (CPython 3.12 oracle)."""


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
with temp_cwd():
    script = 'script.py'
    with open(script, 'w') as f:
        f.write("print(f'\\{1}')")
    _, stdout, stderr = assert_python_ok(script)

    assert b'\\1' in stdout

    assert len(stderr.strip().splitlines()) == 2
print("TestCase::test_syntax_warning_infinite_recursion_in_file: ok")
