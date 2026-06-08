# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "fnmatch_test_case__test_char_set"
# subject = "cpython.test_fnmatch.FnmatchTestCase.test_char_set"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fnmatch.py::FnmatchTestCase::test_char_set
"""Auto-ported test: FnmatchTestCase::test_char_set (CPython 3.12 oracle)."""


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
check = check_match
tescases = string.ascii_lowercase + string.digits + string.punctuation
for c in tescases:
    check(c, '[az]', c in 'az')
    check(c, '[!az]', c not in 'az')
for c in tescases:
    check(c, '[AZ]', c in 'az' and ignorecase)
    check(c, '[!AZ]', c not in 'az' or not ignorecase)
for c in string.ascii_uppercase:
    check(c, '[az]', c in 'AZ' and ignorecase)
    check(c, '[!az]', c not in 'AZ' or not ignorecase)
for c in tescases:
    check(c, '[aa]', c == 'a')
for c in tescases:
    check(c, '[^az]', c in '^az')
    check(c, '[[az]', c in '[az')
    check(c, '[!]]', c != ']')
check('[', '[')
check('[]', '[]')
check('[!', '[!')
check('[!]', '[!]')
print("FnmatchTestCase::test_char_set: ok")
