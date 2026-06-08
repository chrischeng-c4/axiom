# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "key_ordering_test__test_ordered_dict_reader"
# subject = "cpython.test_csv.KeyOrderingTest.test_ordered_dict_reader"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_csv.py::KeyOrderingTest::test_ordered_dict_reader
"""Auto-ported test: KeyOrderingTest::test_ordered_dict_reader (CPython 3.12 oracle)."""


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
data = dedent('            FirstName,LastName\n            Eric,Idle\n            Graham,Chapman,Over1,Over2\n\n            Under1\n            John,Cleese\n        ').splitlines()

assert list(csv.DictReader(data)) == [OrderedDict([('FirstName', 'Eric'), ('LastName', 'Idle')]), OrderedDict([('FirstName', 'Graham'), ('LastName', 'Chapman'), (None, ['Over1', 'Over2'])]), OrderedDict([('FirstName', 'Under1'), ('LastName', None)]), OrderedDict([('FirstName', 'John'), ('LastName', 'Cleese')])]

assert list(csv.DictReader(data, restkey='OtherInfo')) == [OrderedDict([('FirstName', 'Eric'), ('LastName', 'Idle')]), OrderedDict([('FirstName', 'Graham'), ('LastName', 'Chapman'), ('OtherInfo', ['Over1', 'Over2'])]), OrderedDict([('FirstName', 'Under1'), ('LastName', None)]), OrderedDict([('FirstName', 'John'), ('LastName', 'Cleese')])]
del data[0]

assert list(csv.DictReader(data, fieldnames=['fname', 'lname'])) == [OrderedDict([('fname', 'Eric'), ('lname', 'Idle')]), OrderedDict([('fname', 'Graham'), ('lname', 'Chapman'), (None, ['Over1', 'Over2'])]), OrderedDict([('fname', 'Under1'), ('lname', None)]), OrderedDict([('fname', 'John'), ('lname', 'Cleese')])]
print("KeyOrderingTest::test_ordered_dict_reader: ok")
