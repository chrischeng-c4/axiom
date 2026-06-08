# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ucn"
# dimension = "behavior"
# case = "unicode_names_test__test_named_sequences_sample"
# subject = "cpython.test_ucn.UnicodeNamesTest.test_named_sequences_sample"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ucn.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ucn.py::UnicodeNamesTest::test_named_sequences_sample
"""Auto-ported test: UnicodeNamesTest::test_named_sequences_sample (CPython 3.12 oracle)."""


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
sequences = [('LATIN SMALL LETTER R WITH TILDE', 'r̃'), ('TAMIL SYLLABLE SAI', 'ஸை'), ('TAMIL SYLLABLE MOO', 'மோ'), ('TAMIL SYLLABLE NNOO', 'ணோ'), ('TAMIL CONSONANT KSS', 'க்ஷ்')]
for seqname, codepoints in sequences:

    assert unicodedata.lookup(seqname) == codepoints
    try:
        checkletter(seqname, None)
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
    try:
        unicodedata.ucd_3_2_0.lookup(seqname)
        raise AssertionError('expected KeyError')
    except KeyError:
        pass
print("UnicodeNamesTest::test_named_sequences_sample: ok")
