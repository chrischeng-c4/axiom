# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "eof"
# dimension = "behavior"
# case = "eof_test_case__test_eofs_with_file"
# subject = "cpython.test_eof.EOFTestCase.test_EOFS_with_file"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_eof.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_eof.py::EOFTestCase::test_EOFS_with_file
"""Auto-ported test: EOFTestCase::test_EOFS_with_file (CPython 3.12 oracle)."""


import sys
from codecs import BOM_UTF8
from test import support
from test.support import os_helper
from test.support import script_helper
from test.support import warnings_helper
import unittest


'test script for a few new invalid token catches'


# --- test body ---
expect = '(<string>, line 1)'
with os_helper.temp_dir() as temp_dir:
    file_name = script_helper.make_script(temp_dir, 'foo', "ä = '''thîs is \na \ntest")
    rc, out, err = script_helper.assert_python_failure('-X', 'utf8', file_name)
    err = err.decode().splitlines()

    assert err[-3:] == ["    ä = '''thîs is ", '        ^', 'SyntaxError: unterminated triple-quoted string literal (detected at line 3)']
    file_name = script_helper.make_script(temp_dir, 'foo', "ä = '''thîs is \na \ntest".encode())
    rc, out, err = script_helper.assert_python_failure('-X', 'utf8', file_name)
    err = err.decode().splitlines()

    assert err[-3:] == ["    ä = '''thîs is ", '        ^', 'SyntaxError: unterminated triple-quoted string literal (detected at line 3)']
    file_name = script_helper.make_script(temp_dir, 'foo', BOM_UTF8 + "ä = '''thîs is \na \ntest".encode())
    rc, out, err = script_helper.assert_python_failure('-X', 'utf8', file_name)
    err = err.decode().splitlines()

    assert err[-3:] == ["    ä = '''thîs is ", '        ^', 'SyntaxError: unterminated triple-quoted string literal (detected at line 3)']
    file_name = script_helper.make_script(temp_dir, 'foo', "# coding: latin1\nä = '''thîs is \na \ntest".encode('latin1'))
    rc, out, err = script_helper.assert_python_failure('-X', 'utf8', file_name)
    err = err.decode().splitlines()

    assert err[-3:] == ["    ä = '''thîs is ", '        ^', 'SyntaxError: unterminated triple-quoted string literal (detected at line 4)']
print("EOFTestCase::test_EOFS_with_file: ok")
