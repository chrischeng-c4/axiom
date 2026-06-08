# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_unix__test_simple_writer"
# subject = "cpython.test_csv.TestDialectUnix.test_simple_writer"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_csv.py::TestDialectUnix::test_simple_writer
"""Auto-ported test: TestDialectUnix::test_simple_writer (CPython 3.12 oracle)."""


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
dialect = 'unix'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
writerAssertEqual([[1, 'abc def', 'abc']], '"1","abc def","abc"\n')
print("TestDialectUnix::test_simple_writer: ok")
