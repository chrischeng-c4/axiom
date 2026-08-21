# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ucn"
# dimension = "behavior"
# case = "unicode_names_test__test_strict_error_handling"
# subject = "cpython.test_ucn.UnicodeNamesTest.test_strict_error_handling"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ucn.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ucn.py::UnicodeNamesTest::test_strict_error_handling
"""Auto-ported test: UnicodeNamesTest::test_strict_error_handling (CPython 3.12 oracle)."""


import ast
import unittest
import unicodedata
import urllib.error
from test import support
from http.client import HTTPException


' Test script for the Unicode implementation.\n\nWritten by Bill Tutt.\nModified for Python 2.0 by Fredrik Lundh (fredrik@pythonware.com)\n\n(c) Copyright CNRI, All Rights Reserved. NO WARRANTY.\n\n'

try:
    from _testcapi import INT_MAX, PY_SSIZE_T_MAX, UINT_MAX
except ImportError:
    INT_MAX = PY_SSIZE_T_MAX = UINT_MAX = 2 ** 64 - 1


# --- test body ---

try:
    str(b'\\N{blah}', 'unicode-escape', 'strict')
    raise AssertionError('expected UnicodeError')
except UnicodeError:
    pass

try:
    str(bytes('\\N{%s}' % ('x' * 100000), 'ascii'), 'unicode-escape', 'strict')
    raise AssertionError('expected UnicodeError')
except UnicodeError:
    pass

try:
    str(b'\\N{SPACE', 'unicode-escape', 'strict')
    raise AssertionError('expected UnicodeError')
except UnicodeError:
    pass

try:
    str(b'\\NSPACE', 'unicode-escape', 'strict')
    raise AssertionError('expected UnicodeError')
except UnicodeError:
    pass
print("UnicodeNamesTest::test_strict_error_handling: ok")
