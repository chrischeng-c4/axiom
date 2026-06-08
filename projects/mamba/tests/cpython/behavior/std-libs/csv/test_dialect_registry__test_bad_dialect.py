# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_registry__test_bad_dialect"
# subject = "cpython.test_csv.TestDialectRegistry.test_bad_dialect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectRegistry::test_bad_dialect
"""Auto-ported test: TestDialectRegistry::test_bad_dialect (CPython 3.12 oracle)."""


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

try:
    csv.reader([], bad_attr=0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    csv.reader([], delimiter=None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    csv.reader([], quoting=-1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    csv.reader([], quoting=100)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestDialectRegistry::test_bad_dialect: ok")
