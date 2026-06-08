# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "filter_test_case__test_sep"
# subject = "cpython.test_fnmatch.FilterTestCase.test_sep"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fnmatch.py::FilterTestCase::test_sep
"""Auto-ported test: FilterTestCase::test_sep (CPython 3.12 oracle)."""


import unittest
import os
import string
import warnings
from fnmatch import fnmatch, fnmatchcase, translate, filter


'Test cases for the fnmatch module.'


# --- test body ---
normsep = os.path.normcase('\\') == os.path.normcase('/')

assert filter(['usr/bin', 'usr', 'usr\\lib'], 'usr/*') == (['usr/bin', 'usr\\lib'] if normsep else ['usr/bin'])

assert filter(['usr/bin', 'usr', 'usr\\lib'], 'usr\\*') == (['usr/bin', 'usr\\lib'] if normsep else ['usr\\lib'])
print("FilterTestCase::test_sep: ok")
