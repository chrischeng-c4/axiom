# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "filter_test_case__test_case"
# subject = "cpython.test_fnmatch.FilterTestCase.test_case"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fnmatch.py::FilterTestCase::test_case
"""Auto-ported test: FilterTestCase::test_case (CPython 3.12 oracle)."""


import unittest
import os
import string
import warnings
from fnmatch import fnmatch, fnmatchcase, translate, filter


'Test cases for the fnmatch module.'


# --- test body ---
ignorecase = os.path.normcase('P') == os.path.normcase('p')

assert filter(['Test.py', 'Test.rb', 'Test.PL'], '*.p*') == (['Test.py', 'Test.PL'] if ignorecase else ['Test.py'])

assert filter(['Test.py', 'Test.rb', 'Test.PL'], '*.P*') == (['Test.py', 'Test.PL'] if ignorecase else ['Test.PL'])
print("FilterTestCase::test_case: ok")
