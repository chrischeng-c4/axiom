# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "filter_test_case__test_mix_bytes_str"
# subject = "cpython.test_fnmatch.FilterTestCase.test_mix_bytes_str"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fnmatch.py::FilterTestCase::test_mix_bytes_str
"""Auto-ported test: FilterTestCase::test_mix_bytes_str (CPython 3.12 oracle)."""


import unittest
import os
import string
import warnings
from fnmatch import fnmatch, fnmatchcase, translate, filter


'Test cases for the fnmatch module.'


# --- test body ---

try:
    filter(['test'], b'*')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    filter([b'test'], '*')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("FilterTestCase::test_mix_bytes_str: ok")
