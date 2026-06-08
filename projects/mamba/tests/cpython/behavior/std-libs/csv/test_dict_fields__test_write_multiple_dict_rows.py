# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_write_multiple_dict_rows"
# subject = "cpython.test_csv.TestDictFields.test_write_multiple_dict_rows"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_write_multiple_dict_rows
"""Auto-ported test: TestDictFields::test_write_multiple_dict_rows (CPython 3.12 oracle)."""


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
fileobj = StringIO()
writer = csv.DictWriter(fileobj, fieldnames=['f1', 'f2', 'f3'])
writer.writeheader()

assert fileobj.getvalue() == 'f1,f2,f3\r\n'
writer.writerows([{'f1': 1, 'f2': 'abc', 'f3': 'f'}, {'f1': 2, 'f2': 5, 'f3': 'xyz'}])

assert fileobj.getvalue() == 'f1,f2,f3\r\n1,abc,f\r\n2,5,xyz\r\n'
print("TestDictFields::test_write_multiple_dict_rows: ok")
