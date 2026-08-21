# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_array_writes__test_char_write"
# subject = "cpython.test_csv.TestArrayWrites.test_char_write"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_csv.py::TestArrayWrites::test_char_write
"""Auto-ported test: TestArrayWrites::test_char_write (CPython 3.12 oracle)."""


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
import array, string
a = array.array('u', string.ascii_letters)
with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
    writer = csv.writer(fileobj, dialect='excel')
    writer.writerow(a)
    expected = ','.join(a) + '\r\n'
    fileobj.seek(0)

    assert fileobj.read() == expected
print("TestArrayWrites::test_char_write: ok")
