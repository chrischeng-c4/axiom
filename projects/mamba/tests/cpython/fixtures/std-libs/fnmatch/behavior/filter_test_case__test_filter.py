# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "filter_test_case__test_filter"
# subject = "cpython.test_fnmatch.FilterTestCase.test_filter"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fnmatch.py::FilterTestCase::test_filter
"""Auto-ported test: FilterTestCase::test_filter (CPython 3.12 oracle)."""


import unittest
import os
import string
import warnings
from fnmatch import fnmatch, fnmatchcase, translate, filter


'Test cases for the fnmatch module.'


# --- test body ---

assert filter(['Python', 'Ruby', 'Perl', 'Tcl'], 'P*') == ['Python', 'Perl']

assert filter([b'Python', b'Ruby', b'Perl', b'Tcl'], b'P*') == [b'Python', b'Perl']
print("FilterTestCase::test_filter: ok")
