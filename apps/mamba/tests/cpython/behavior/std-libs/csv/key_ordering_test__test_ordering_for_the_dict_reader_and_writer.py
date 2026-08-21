# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "key_ordering_test__test_ordering_for_the_dict_reader_and_writer"
# subject = "cpython.test_csv.KeyOrderingTest.test_ordering_for_the_dict_reader_and_writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::KeyOrderingTest::test_ordering_for_the_dict_reader_and_writer
"""Auto-ported test: KeyOrderingTest::test_ordering_for_the_dict_reader_and_writer (CPython 3.12 oracle)."""


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
resultset = set()
for keys in permutations('abcde'):
    with TemporaryFile('w+', newline='', encoding='utf-8') as fileobject:
        dw = csv.DictWriter(fileobject, keys)
        dw.writeheader()
        fileobject.seek(0)
        dr = csv.DictReader(fileobject)
        kt = tuple(dr.fieldnames)

        assert keys == kt
        resultset.add(kt)

assert len(resultset) == 120
print("KeyOrderingTest::test_ordering_for_the_dict_reader_and_writer: ok")
