# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_unicode__test_unicode_write"
# subject = "cpython.test_csv.TestUnicode.test_unicode_write"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestUnicode::test_unicode_write
"""Auto-ported test: TestUnicode::test_unicode_write (CPython 3.12 oracle)."""


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
names = ['Martin von Löwis', 'Marc André Lemburg', 'Guido van Rossum', 'François Pinard']
with TemporaryFile('w+', newline='', encoding='utf-8') as fileobj:
    writer = csv.writer(fileobj)
    writer.writerow(names)
    expected = ','.join(names) + '\r\n'
    fileobj.seek(0)

    assert fileobj.read() == expected
print("TestUnicode::test_unicode_write: ok")
