# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "fnmatch_test_case__test_mix_bytes_str"
# subject = "cpython.test_fnmatch.FnmatchTestCase.test_mix_bytes_str"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_fnmatch.py::FnmatchTestCase::test_mix_bytes_str
"""Auto-ported test: FnmatchTestCase::test_mix_bytes_str (CPython 3.12 oracle)."""


import unittest
import os
import string
import warnings
from fnmatch import fnmatch, fnmatchcase, translate, filter


'Test cases for the fnmatch module.'


# --- test body ---

try:
    fnmatch('test', b'*')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    fnmatch(b'test', '*')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    fnmatchcase('test', b'*')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    fnmatchcase(b'test', '*')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("FnmatchTestCase::test_mix_bytes_str: ok")
