# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_write_simple_dict"
# subject = "cpython.test_csv.TestDictFields.test_write_simple_dict"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_write_simple_dict
"""Auto-ported test: TestDictFields::test_write_simple_dict (CPython 3.12 oracle)."""


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
    writer = csv.DictWriter(fileobj, fieldnames=['f1', 'f2', 'f3'])
    writer.writeheader()
    fileobj.seek(0)

    assert fileobj.readline() == 'f1,f2,f3\r\n'
    writer.writerow({'f1': 10, 'f3': 'abc'})
    fileobj.seek(0)
    fileobj.readline()

    assert fileobj.read() == '10,,abc\r\n'
print("TestDictFields::test_write_simple_dict: ok")
