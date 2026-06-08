# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_read_multi"
# subject = "cpython.test_csv.TestDictFields.test_read_multi"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_read_multi
"""Auto-ported test: TestDictFields::test_read_multi (CPython 3.12 oracle)."""


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
sample = ['2147483648,43.0e12,17,abc,def\r\n', '147483648,43.0e2,17,abc,def\r\n', '47483648,43.0,170,abc,def\r\n']
reader = csv.DictReader(sample, fieldnames='i1 float i2 s1 s2'.split())

assert next(reader) == {'i1': '2147483648', 'float': '43.0e12', 'i2': '17', 's1': 'abc', 's2': 'def'}
print("TestDictFields::test_read_multi: ok")
