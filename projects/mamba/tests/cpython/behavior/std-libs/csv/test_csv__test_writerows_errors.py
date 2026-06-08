# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_csv__test_writerows_errors"
# subject = "cpython.test_csv.Test_Csv.test_writerows_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_csv.py::Test_Csv::test_writerows_errors
"""Auto-ported test: Test_Csv::test_writerows_errors (CPython 3.12 oracle)."""


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
with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
    writer = csv.writer(fileobj)

    try:
        writer.writerows(None)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        writer.writerows(BadIterable())
        raise AssertionError('expected OSError')
    except OSError:
        pass
print("Test_Csv::test_writerows_errors: ok")
