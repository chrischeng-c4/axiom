# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_read_dict_fields"
# subject = "cpython.test_csv.TestDictFields.test_read_dict_fields"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_read_dict_fields
"""Auto-ported test: TestDictFields::test_read_dict_fields (CPython 3.12 oracle)."""


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
with TemporaryFile('w+', encoding='utf-8') as fileobj:
    fileobj.write('1,2,abc\r\n')
    fileobj.seek(0)
    reader = csv.DictReader(fileobj, fieldnames=['f1', 'f2', 'f3'])

    assert next(reader) == {'f1': '1', 'f2': '2', 'f3': 'abc'}
print("TestDictFields::test_read_dict_fields: ok")
