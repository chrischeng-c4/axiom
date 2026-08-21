# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ucn"
# dimension = "behavior"
# case = "unicode_names_test__test_ascii_letters"
# subject = "cpython.test_ucn.UnicodeNamesTest.test_ascii_letters"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ucn.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ucn.py::UnicodeNamesTest::test_ascii_letters
"""Auto-ported test: UnicodeNamesTest::test_ascii_letters (CPython 3.12 oracle)."""


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
for char in ''.join(map(chr, range(ord('a'), ord('z')))):
    name = 'LATIN SMALL LETTER %s' % char.upper()
    code = unicodedata.lookup(name)

    assert unicodedata.name(code) == name
print("UnicodeNamesTest::test_ascii_letters: ok")
