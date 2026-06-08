# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_validity__test_delimiter"
# subject = "cpython.test_csv.TestDialectValidity.test_delimiter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectValidity::test_delimiter
"""Auto-ported test: TestDialectValidity::test_delimiter (CPython 3.12 oracle)."""


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
class mydialect(csv.Dialect):
    delimiter = ';'
    escapechar = '\\'
    doublequote = False
    skipinitialspace = True
    lineterminator = '\r\n'
    quoting = csv.QUOTE_NONE
d = mydialect()

assert d.delimiter == ';'
mydialect.delimiter = ':::'
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == '"delimiter" must be a 1-character string'
mydialect.delimiter = ''
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == '"delimiter" must be a 1-character string'
mydialect.delimiter = b','
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == '"delimiter" must be string, not bytes'
mydialect.delimiter = 4
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == '"delimiter" must be string, not int'
mydialect.delimiter = None
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == '"delimiter" must be string, not NoneType'
print("TestDialectValidity::test_delimiter: ok")
