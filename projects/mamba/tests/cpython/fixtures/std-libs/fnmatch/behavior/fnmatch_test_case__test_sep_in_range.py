# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "fnmatch_test_case__test_sep_in_range"
# subject = "cpython.test_fnmatch.FnmatchTestCase.test_sep_in_range"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fnmatch.py::FnmatchTestCase::test_sep_in_range
"""Auto-ported test: FnmatchTestCase::test_sep_in_range (CPython 3.12 oracle)."""


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
normsep = os.path.normcase('\\') == os.path.normcase('/')
check = check_match
check('a/b', 'a[.-0]b', not normsep)
check('a\\b', 'a[.-0]b', False)
check('a\\b', 'a[Z-^]b', not normsep)
check('a/b', 'a[Z-^]b', False)
check('a/b', 'a[/-0]b', not normsep)
check('a\\b', 'a[/-0]b', False)
check('a[/-0]b', 'a[/-0]b', False)
check('a[\\-0]b', 'a[/-0]b', False)
check('a/b', 'a[.-/]b')
check('a\\b', 'a[.-/]b', normsep)
check('a[.-/]b', 'a[.-/]b', False)
check('a[.-\\]b', 'a[.-/]b', False)
check('a\\b', 'a[\\-^]b')
check('a/b', 'a[\\-^]b', normsep)
check('a[\\-^]b', 'a[\\-^]b', False)
check('a[/-^]b', 'a[\\-^]b', False)
check('a\\b', 'a[Z-\\]b', not normsep)
check('a/b', 'a[Z-\\]b', False)
check('a[Z-\\]b', 'a[Z-\\]b', False)
check('a[Z-/]b', 'a[Z-\\]b', False)
print("FnmatchTestCase::test_sep_in_range: ok")
