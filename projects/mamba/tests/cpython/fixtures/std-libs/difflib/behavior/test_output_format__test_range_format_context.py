# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_output_format__test_range_format_context"
# subject = "cpython.test_difflib.TestOutputFormat.test_range_format_context"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_difflib.py::TestOutputFormat::test_range_format_context
"""Auto-ported test: TestOutputFormat::test_range_format_context (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
spec = '           The range of lines in file1 shall be written in the following format\n           if the range contains two or more lines:\n               "*** %d,%d ****\n", <beginning line number>, <ending line number>\n           and the following format otherwise:\n               "*** %d ****\n", <ending line number>\n           The ending line number of an empty range shall be the number of the preceding line,\n           or 0 if the range is at the start of the file.\n\n           Next, the range of lines in file2 shall be written in the following format\n           if the range contains two or more lines:\n               "--- %d,%d ----\n", <beginning line number>, <ending line number>\n           and the following format otherwise:\n               "--- %d ----\n", <ending line number>\n        '
fmt = difflib._format_range_context

assert fmt(3, 3) == '3'

assert fmt(3, 4) == '4'

assert fmt(3, 5) == '4,5'

assert fmt(3, 6) == '4,6'

assert fmt(0, 0) == '0'
print("TestOutputFormat::test_range_format_context: ok")
