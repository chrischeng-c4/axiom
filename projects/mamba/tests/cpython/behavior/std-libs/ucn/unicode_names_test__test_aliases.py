# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ucn"
# dimension = "behavior"
# case = "unicode_names_test__test_aliases"
# subject = "cpython.test_ucn.UnicodeNamesTest.test_aliases"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ucn.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ucn.py::UnicodeNamesTest::test_aliases
"""Auto-ported test: UnicodeNamesTest::test_aliases (CPython 3.12 oracle)."""


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
aliases = [('LATIN CAPITAL LETTER GHA', 418), ('LATIN SMALL LETTER GHA', 419), ('KANNADA LETTER LLLA', 3294), ('LAO LETTER FO FON', 3741), ('LAO LETTER FO FAY', 3743), ('LAO LETTER RO', 3747), ('LAO LETTER LO', 3749), ('TIBETAN MARK BKA- SHOG GI MGO RGYAN', 4048), ('YI SYLLABLE ITERATION MARK', 40981), ('PRESENTATION FORM FOR VERTICAL RIGHT WHITE LENTICULAR BRACKET', 65048), ('BYZANTINE MUSICAL SYMBOL FTHORA SKLIRON CHROMA VASIS', 118981)]
for alias, codepoint in aliases:
    checkletter(alias, chr(codepoint))
    name = unicodedata.name(chr(codepoint))

    assert name != alias

    assert unicodedata.lookup(alias) == unicodedata.lookup(name)
    try:
        unicodedata.ucd_3_2_0.lookup(alias)
        raise AssertionError('expected KeyError')
    except KeyError:
        pass
print("UnicodeNamesTest::test_aliases: ok")
