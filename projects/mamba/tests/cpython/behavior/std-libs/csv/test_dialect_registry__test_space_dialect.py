# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_registry__test_space_dialect"
# subject = "cpython.test_csv.TestDialectRegistry.test_space_dialect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectRegistry::test_space_dialect
"""Auto-ported test: TestDialectRegistry::test_space_dialect (CPython 3.12 oracle)."""


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
class space(csv.excel):
    delimiter = ' '
    quoting = csv.QUOTE_NONE
    escapechar = '\\'
with TemporaryFile('w+', encoding='utf-8') as fileobj:
    fileobj.write('abc   def\nc1ccccc1 benzene\n')
    fileobj.seek(0)
    reader = csv.reader(fileobj, dialect=space())

    assert next(reader) == ['abc', '', '', 'def']

    assert next(reader) == ['c1ccccc1', 'benzene']
print("TestDialectRegistry::test_space_dialect: ok")
