# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_read_short"
# subject = "cpython.test_csv.TestDictFields.test_read_short"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_read_short
"""Auto-ported test: TestDictFields::test_read_short (CPython 3.12 oracle)."""


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
    fileobj.write('1,2,abc,4,5,6\r\n1,2,abc\r\n')
    fileobj.seek(0)
    reader = csv.DictReader(fileobj, fieldnames='1 2 3 4 5 6'.split(), restval='DEFAULT')

    assert next(reader) == {'1': '1', '2': '2', '3': 'abc', '4': '4', '5': '5', '6': '6'}

    assert next(reader) == {'1': '1', '2': '2', '3': 'abc', '4': 'DEFAULT', '5': 'DEFAULT', '6': 'DEFAULT'}
print("TestDictFields::test_read_short: ok")
