# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_validity__test_invalid_chars"
# subject = "cpython.test_csv.TestDialectValidity.test_invalid_chars"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_csv.py::TestDialectValidity::test_invalid_chars
"""Auto-ported test: TestDialectValidity::test_invalid_chars (CPython 3.12 oracle)."""


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
def create_invalid(field_name, value):

    class mydialect(csv.Dialect):
        pass
    setattr(mydialect, field_name, value)
    d = mydialect()
for field_name in ('delimiter', 'escapechar', 'quotechar'):

    try:
        create_invalid(field_name, '')
        raise AssertionError('expected csv.Error')
    except csv.Error:
        pass

    try:
        create_invalid(field_name, 'abc')
        raise AssertionError('expected csv.Error')
    except csv.Error:
        pass

    try:
        create_invalid(field_name, b'x')
        raise AssertionError('expected csv.Error')
    except csv.Error:
        pass

    try:
        create_invalid(field_name, 5)
        raise AssertionError('expected csv.Error')
    except csv.Error:
        pass
print("TestDialectValidity::test_invalid_chars: ok")
