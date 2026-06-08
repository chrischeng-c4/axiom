# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "fnmatch_test_case__test_sep"
# subject = "cpython.test_fnmatch.FnmatchTestCase.test_sep"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fnmatch.py::FnmatchTestCase::test_sep
"""Auto-ported test: FnmatchTestCase::test_sep (CPython 3.12 oracle)."""


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
check('usr/bin', 'usr/bin')
check('usr\\bin', 'usr/bin', normsep)
check('usr/bin', 'usr\\bin', normsep)
check('usr\\bin', 'usr\\bin')
print("FnmatchTestCase::test_sep: ok")
