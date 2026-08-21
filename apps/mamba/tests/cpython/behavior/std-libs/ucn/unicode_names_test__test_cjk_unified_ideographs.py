# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ucn"
# dimension = "behavior"
# case = "unicode_names_test__test_cjk_unified_ideographs"
# subject = "cpython.test_ucn.UnicodeNamesTest.test_cjk_unified_ideographs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ucn.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ucn.py::UnicodeNamesTest::test_cjk_unified_ideographs
"""Auto-ported test: UnicodeNamesTest::test_cjk_unified_ideographs (CPython 3.12 oracle)."""


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
checkletter('CJK UNIFIED IDEOGRAPH-3400', '㐀')
checkletter('CJK UNIFIED IDEOGRAPH-4DB5', '䶵')
checkletter('CJK UNIFIED IDEOGRAPH-4E00', '一')
checkletter('CJK UNIFIED IDEOGRAPH-9FCB', '鿋')
checkletter('CJK UNIFIED IDEOGRAPH-20000', '𠀀')
checkletter('CJK UNIFIED IDEOGRAPH-2A6D6', '𪛖')
checkletter('CJK UNIFIED IDEOGRAPH-2A700', '𪜀')
checkletter('CJK UNIFIED IDEOGRAPH-2B734', '𫜴')
checkletter('CJK UNIFIED IDEOGRAPH-2B740', '𫝀')
checkletter('CJK UNIFIED IDEOGRAPH-2B81D', '𫠝')
checkletter('CJK UNIFIED IDEOGRAPH-3134A', '𱍊')
print("UnicodeNamesTest::test_cjk_unified_ideographs: ok")
