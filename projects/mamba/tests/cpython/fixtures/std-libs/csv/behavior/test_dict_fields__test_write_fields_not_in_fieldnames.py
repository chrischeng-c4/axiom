# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_write_fields_not_in_fieldnames"
# subject = "cpython.test_csv.TestDictFields.test_write_fields_not_in_fieldnames"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_write_fields_not_in_fieldnames
"""Auto-ported test: TestDictFields::test_write_fields_not_in_fieldnames (CPython 3.12 oracle)."""


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
    try:
        writer.writerow({'f4': 10, 'f2': 'spam', 1: 'abc'})
        raise AssertionError('expected ValueError')
    except ValueError as _aR_e:
        import types as _types_aR
        cx = _types_aR.SimpleNamespace(exception=_aR_e)
    exception = str(cx.exception)

    assert 'fieldnames' in exception

    assert "'f4'" in exception

    assert "'f2'" not in exception

    assert '1' in exception
print("TestDictFields::test_write_fields_not_in_fieldnames: ok")
