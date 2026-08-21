# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "fnmatch_test_case__test_fnmatch"
# subject = "cpython.test_fnmatch.FnmatchTestCase.test_fnmatch"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fnmatch.py::FnmatchTestCase::test_fnmatch
"""Auto-ported test: FnmatchTestCase::test_fnmatch (CPython 3.12 oracle)."""


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
check('abc', 'abc')
check('abc', '?*?')
check('abc', '???*')
check('abc', '*???')
check('abc', '???')
check('abc', '*')
check('abc', 'ab[cd]')
check('abc', 'ab[!de]')
check('abc', 'ab[de]', False)
check('a', '??', False)
check('a', 'b', False)
check('\\', '[\\]')
check('a', '[!\\]')
check('\\', '[!\\]', False)
check('foo\nbar', 'foo*')
check('foo\nbar\n', 'foo*')
check('\nfoo', 'foo*', False)
check('\n', '*')
print("FnmatchTestCase::test_fnmatch: ok")
