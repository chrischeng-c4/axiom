# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ucn"
# dimension = "behavior"
# case = "unicode_names_test__test_errors"
# subject = "cpython.test_ucn.UnicodeNamesTest.test_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ucn.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ucn.py::UnicodeNamesTest::test_errors
"""Auto-ported test: UnicodeNamesTest::test_errors (CPython 3.12 oracle)."""


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
    unicodedata.name()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    unicodedata.name('xx')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    unicodedata.lookup()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    unicodedata.lookup('unknown')
    raise AssertionError('expected KeyError')
except KeyError:
    pass
print("UnicodeNamesTest::test_errors: ok")
