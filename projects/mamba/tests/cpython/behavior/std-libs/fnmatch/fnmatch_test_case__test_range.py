# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "fnmatch_test_case__test_range"
# subject = "cpython.test_fnmatch.FnmatchTestCase.test_range"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fnmatch.py::FnmatchTestCase::test_range
"""Auto-ported test: FnmatchTestCase::test_range (CPython 3.12 oracle)."""


import unittest
import os
import string
import warnings
from fnmatch import fnmatch, fnmatchcase, translate, filter


'Test cases for the fnmatch module.'


# --- test body ---
def check_match(filename, pattern, should_match=True, fn=fnmatch):
    if should_match:

        assert fn(filename, pattern)
    else:

        assert not fn(filename, pattern)
ignorecase = os.path.normcase('ABC') == os.path.normcase('abc')
normsep = os.path.normcase('\\') == os.path.normcase('/')
check = check_match
tescases = string.ascii_lowercase + string.digits + string.punctuation
for c in tescases:
    check(c, '[b-d]', c in 'bcd')
    check(c, '[!b-d]', c not in 'bcd')
    check(c, '[b-dx-z]', c in 'bcdxyz')
    check(c, '[!b-dx-z]', c not in 'bcdxyz')
for c in tescases:
    check(c, '[B-D]', c in 'bcd' and ignorecase)
    check(c, '[!B-D]', c not in 'bcd' or not ignorecase)
for c in string.ascii_uppercase:
    check(c, '[b-d]', c in 'BCD' and ignorecase)
    check(c, '[!b-d]', c not in 'BCD' or not ignorecase)
for c in tescases:
    check(c, '[b-b]', c == 'b')
for c in tescases:
    check(c, '[!-#]', c not in '-#')
    check(c, '[!--.]', c not in '-.')
    check(c, '[^-`]', c in '^_`')
    if not (normsep and c == '/'):
        check(c, '[[-^]', c in '[\\]^')
        check(c, '[\\-^]', c in '\\]^')
    check(c, '[b-]', c in '-b')
    check(c, '[!b-]', c not in '-b')
    check(c, '[-b]', c in '-b')
    check(c, '[!-b]', c not in '-b')
    check(c, '[-]', c in '-')
    check(c, '[!-]', c not in '-')
for c in tescases:
    check(c, '[d-b]', False)
    check(c, '[!d-b]', True)
    check(c, '[d-bx-z]', c in 'xyz')
    check(c, '[!d-bx-z]', c not in 'xyz')
    check(c, '[d-b^-`]', c in '^_`')
    if not (normsep and c == '/'):
        check(c, '[d-b[-^]', c in '[\\]^')
print("FnmatchTestCase::test_range: ok")
