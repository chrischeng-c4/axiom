# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_registry__test_register_kwargs"
# subject = "cpython.test_csv.TestDialectRegistry.test_register_kwargs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_csv.py::TestDialectRegistry::test_register_kwargs
"""Auto-ported test: TestDialectRegistry::test_register_kwargs (CPython 3.12 oracle)."""


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
def compare_dialect_123(expected, *writeargs, **kwwriteargs):
    with TemporaryFile('w+', newline='', encoding='utf-8') as fileobj:
        writer = csv.writer(fileobj, *writeargs, **kwwriteargs)
        writer.writerow([1, 2, 3])
        fileobj.seek(0)

        assert fileobj.read() == expected
name = 'fedcba'
csv.register_dialect(name, delimiter=';')
pass

assert csv.get_dialect(name).delimiter == ';'

assert [['X', 'Y', 'Z']] == list(csv.reader(['X;Y;Z'], name))
print("TestDialectRegistry::test_register_kwargs: ok")
