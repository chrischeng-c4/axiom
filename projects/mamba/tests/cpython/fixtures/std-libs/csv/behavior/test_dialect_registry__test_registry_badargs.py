# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_registry__test_registry_badargs"
# subject = "cpython.test_csv.TestDialectRegistry.test_registry_badargs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_csv.py::TestDialectRegistry::test_registry_badargs
"""Auto-ported test: TestDialectRegistry::test_registry_badargs (CPython 3.12 oracle)."""


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

try:
    csv.list_dialects(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    csv.get_dialect()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    csv.get_dialect(None)
    raise AssertionError('expected csv.Error')
except csv.Error:
    pass

try:
    csv.get_dialect('nonesuch')
    raise AssertionError('expected csv.Error')
except csv.Error:
    pass

try:
    csv.unregister_dialect()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    csv.unregister_dialect(None)
    raise AssertionError('expected csv.Error')
except csv.Error:
    pass

try:
    csv.unregister_dialect('nonesuch')
    raise AssertionError('expected csv.Error')
except csv.Error:
    pass

try:
    csv.register_dialect(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    csv.register_dialect(None, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    csv.register_dialect('nonesuch', 0, 0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    csv.register_dialect('nonesuch', badargument=None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    csv.register_dialect('nonesuch', quoting=None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    csv.register_dialect([])
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestDialectRegistry::test_registry_badargs: ok")
