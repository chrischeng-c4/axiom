# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "translate_test_case__test_translate"
# subject = "cpython.test_fnmatch.TranslateTestCase.test_translate"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fnmatch.py::TranslateTestCase::test_translate
"""Auto-ported test: TranslateTestCase::test_translate (CPython 3.12 oracle)."""


import unittest
import os
import string
import warnings
from fnmatch import fnmatch, fnmatchcase, translate, filter


'Test cases for the fnmatch module.'


# --- test body ---
import re

assert translate('*') == '(?s:.*)\\Z'

assert translate('?') == '(?s:.)\\Z'

assert translate('a?b*') == '(?s:a.b.*)\\Z'

assert translate('[abc]') == '(?s:[abc])\\Z'

assert translate('[]]') == '(?s:[]])\\Z'

assert translate('[!x]') == '(?s:[^x])\\Z'

assert translate('[^x]') == '(?s:[\\^x])\\Z'

assert translate('[x') == '(?s:\\[x)\\Z'

assert translate('*.txt') == '(?s:.*\\.txt)\\Z'

assert translate('*********') == '(?s:.*)\\Z'

assert translate('A*********') == '(?s:A.*)\\Z'

assert translate('*********A') == '(?s:.*A)\\Z'

assert translate('A*********?[?]?') == '(?s:A.*.[?].)\\Z'
t = translate('**a*a****a')

assert t == '(?s:(?>.*?a)(?>.*?a).*a)\\Z'
r1 = translate('**a**a**a*')
r2 = translate('**b**b**b*')
r3 = translate('*c*c*c*')
fatre = '|'.join([r1, r2, r3])

assert re.match(fatre, 'abaccad')

assert re.match(fatre, 'abxbcab')

assert re.match(fatre, 'cbabcaxc')

assert not re.match(fatre, 'dabccbad')
print("TranslateTestCase::test_translate: ok")
