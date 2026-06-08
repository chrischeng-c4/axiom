# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "fnmatch_test_case__test_case"
# subject = "cpython.test_fnmatch.FnmatchTestCase.test_case"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fnmatch.py::FnmatchTestCase::test_case
"""Auto-ported test: FnmatchTestCase::test_case (CPython 3.12 oracle)."""


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
check('abc', 'abc')
check('AbC', 'abc', ignorecase)
check('abc', 'AbC', ignorecase)
check('AbC', 'AbC')
print("FnmatchTestCase::test_case: ok")
