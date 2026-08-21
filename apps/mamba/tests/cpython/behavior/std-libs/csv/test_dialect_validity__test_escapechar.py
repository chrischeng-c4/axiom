# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_validity__test_escapechar"
# subject = "cpython.test_csv.TestDialectValidity.test_escapechar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectValidity::test_escapechar
"""Auto-ported test: TestDialectValidity::test_escapechar (CPython 3.12 oracle)."""


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

assert d.escapechar == '\\'
mydialect.escapechar = ''
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('"escapechar" must be a 1-character string', str(_aR_e))
mydialect.escapechar = '**'
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('"escapechar" must be a 1-character string', str(_aR_e))
mydialect.escapechar = b'*'
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('"escapechar" must be string or None, not bytes', str(_aR_e))
mydialect.escapechar = 4
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('"escapechar" must be string or None, not int', str(_aR_e))
print("TestDialectValidity::test_escapechar: ok")
