# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_validity__test_quoting"
# subject = "cpython.test_csv.TestDialectValidity.test_quoting"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_csv.py::TestDialectValidity::test_quoting
"""Auto-ported test: TestDialectValidity::test_quoting (CPython 3.12 oracle)."""


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

assert d.quoting == csv.QUOTE_NONE
mydialect.quoting = None

try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error:
    pass
mydialect.quoting = 42
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == 'bad "quoting" value'
mydialect.doublequote = True
mydialect.quoting = csv.QUOTE_ALL
mydialect.quotechar = '"'
d = mydialect()

assert d.quoting == csv.QUOTE_ALL

assert d.quotechar == '"'

assert d.doublequote
mydialect.quotechar = ''
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == '"quotechar" must be a 1-character string'
mydialect.quotechar = "''"
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == '"quotechar" must be a 1-character string'
mydialect.quotechar = 4
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == '"quotechar" must be string or None, not int'
print("TestDialectValidity::test_quoting: ok")
