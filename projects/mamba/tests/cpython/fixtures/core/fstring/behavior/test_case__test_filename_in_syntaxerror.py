# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "fstring"
# dimension = "behavior"
# case = "test_case__test_filename_in_syntaxerror"
# subject = "cpython.test_fstring.TestCase.test_filename_in_syntaxerror"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fstring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fstring.py::TestCase::test_filename_in_syntaxerror
"""Auto-ported test: TestCase::test_filename_in_syntaxerror (CPython 3.12 oracle)."""


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
with temp_cwd() as cwd:
    file_path = os.path.join(cwd, 't.py')
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write('f"{a b}"')
    _, _, stderr = assert_python_failure(file_path, PYTHONIOENCODING='ascii')

assert file_path.encode('ascii', 'backslashreplace') in stderr
print("TestCase::test_filename_in_syntaxerror: ok")
