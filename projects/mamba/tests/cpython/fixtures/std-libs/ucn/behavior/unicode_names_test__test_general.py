# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ucn"
# dimension = "behavior"
# case = "unicode_names_test__test_general"
# subject = "cpython.test_ucn.UnicodeNamesTest.test_general"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ucn.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ucn.py::UnicodeNamesTest::test_general
"""Auto-ported test: UnicodeNamesTest::test_general (CPython 3.12 oracle)."""


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
def checkletter(name, code):
    res = ast.literal_eval('"\\N{%s}"' % name)

    assert res == code
    return res
chars = ['LATIN CAPITAL LETTER T', 'LATIN SMALL LETTER H', 'LATIN SMALL LETTER E', 'SPACE', 'LATIN SMALL LETTER R', 'LATIN CAPITAL LETTER E', 'LATIN SMALL LETTER D', 'SPACE', 'LATIN SMALL LETTER f', 'LATIN CAPITAL LeTtEr o', 'LATIN SMaLl LETTER x', 'SPACE', 'LATIN SMALL LETTER A', 'LATIN SMALL LETTER T', 'LATIN SMALL LETTER E', 'SPACE', 'LATIN SMALL LETTER T', 'LATIN SMALL LETTER H', 'LATIN SMALL LETTER E', 'SpAcE', 'LATIN SMALL LETTER S', 'LATIN SMALL LETTER H', 'LATIN small LETTER e', 'LATIN small LETTER e', 'LATIN SMALL LETTER P', 'FULL STOP']
string = 'The rEd fOx ate the sheep.'

assert ''.join([checkletter(*args) for args in zip(chars, string)]) == string
print("UnicodeNamesTest::test_general: ok")
