# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ucn"
# dimension = "behavior"
# case = "unicode_names_test__test_hangul_syllables"
# subject = "cpython.test_ucn.UnicodeNamesTest.test_hangul_syllables"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ucn.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ucn.py::UnicodeNamesTest::test_hangul_syllables
"""Auto-ported test: UnicodeNamesTest::test_hangul_syllables (CPython 3.12 oracle)."""


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
checkletter('HANGUL SYLLABLE GA', '가')
checkletter('HANGUL SYLLABLE GGWEOSS', '꿨')
checkletter('HANGUL SYLLABLE DOLS', '돐')
checkletter('HANGUL SYLLABLE RYAN', '랸')
checkletter('HANGUL SYLLABLE MWIK', '뮠')
checkletter('HANGUL SYLLABLE BBWAEM', '뾈')
checkletter('HANGUL SYLLABLE SSEOL', '썰')
checkletter('HANGUL SYLLABLE YI', '의')
checkletter('HANGUL SYLLABLE JJYOSS', '쭀')
checkletter('HANGUL SYLLABLE KYEOLS', '켨')
checkletter('HANGUL SYLLABLE PAN', '판')
checkletter('HANGUL SYLLABLE HWEOK', '훸')
checkletter('HANGUL SYLLABLE HIH', '힣')

try:
    unicodedata.name('\ud7a4')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("UnicodeNamesTest::test_hangul_syllables: ok")
