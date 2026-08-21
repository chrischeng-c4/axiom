# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "fnmatch_test_case__test_slow_fnmatch"
# subject = "cpython.test_fnmatch.FnmatchTestCase.test_slow_fnmatch"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fnmatch.py::FnmatchTestCase::test_slow_fnmatch
"""Auto-ported test: FnmatchTestCase::test_slow_fnmatch (CPython 3.12 oracle)."""


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
check = check_match
check('a' * 50, '*a*a*a*a*a*a*a*a*a*a')
check('a' * 50 + 'b', '*a*a*a*a*a*a*a*a*a*a', False)
print("FnmatchTestCase::test_slow_fnmatch: ok")
