# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_csv__test_read_linenum"
# subject = "cpython.test_csv.Test_Csv.test_read_linenum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::Test_Csv::test_read_linenum
"""Auto-ported test: Test_Csv::test_read_linenum (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
r = csv.reader(['line,1', 'line,2', 'line,3'])

assert r.line_num == 0
next(r)

assert r.line_num == 1
next(r)

assert r.line_num == 2
next(r)

assert r.line_num == 3

try:
    next(r)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

assert r.line_num == 3
print("Test_Csv::test_read_linenum: ok")
