# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_write_field_not_in_field_names_raise"
# subject = "cpython.test_csv.TestDictFields.test_write_field_not_in_field_names_raise"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_write_field_not_in_field_names_raise
"""Auto-ported test: TestDictFields::test_write_field_not_in_field_names_raise (CPython 3.12 oracle)."""


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
writer = csv.DictWriter(fileobj, ['f1', 'f2'], extrasaction='raise')
dictrow = {'f0': 0, 'f1': 1, 'f2': 2, 'f3': 3}

try:
    csv.DictWriter.writerow(writer, dictrow)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
writer = csv.DictWriter(fileobj, ['f1', 'f2'], extrasaction='RAISE')

try:
    csv.DictWriter.writerow(writer, dictrow)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("TestDictFields::test_write_field_not_in_field_names_raise: ok")
