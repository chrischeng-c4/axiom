# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "fnmatch_test_case__test_warnings"
# subject = "cpython.test_fnmatch.FnmatchTestCase.test_warnings"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fnmatch.py::FnmatchTestCase::test_warnings
"""Auto-ported test: FnmatchTestCase::test_warnings (CPython 3.12 oracle)."""


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
with warnings.catch_warnings():
    warnings.simplefilter('error', Warning)
    check = check_match
    check('[', '[[]')
    check('&', '[a&&b]')
    check('|', '[a||b]')
    check('~', '[a~~b]')
    check(',', '[a-z+--A-Z]')
    check('.', '[a-z--/A-Z]')
print("FnmatchTestCase::test_warnings: ok")
