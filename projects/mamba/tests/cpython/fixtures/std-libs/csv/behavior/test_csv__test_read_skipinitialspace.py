# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_csv__test_read_skipinitialspace"
# subject = "cpython.test_csv.Test_Csv.test_read_skipinitialspace"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_csv.py::Test_Csv::test_read_skipinitialspace
"""Auto-ported test: Test_Csv::test_read_skipinitialspace (CPython 3.12 oracle)."""


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
def _read_test(input, expect, **kwargs):
    reader = csv.reader(input, **kwargs)
    result = list(reader)

    assert result == expect

def _test_arg_valid(ctor, arg):

    try:
        ctor()
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        ctor(None)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        ctor(arg, bad_attr=0)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        ctor(arg, delimiter=0)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        ctor(arg, delimiter='XX')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        ctor(arg, 'foo')
        raise AssertionError('expected csv.Error')
    except csv.Error:
        pass

    try:
        ctor(arg, delimiter=None)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        ctor(arg, delimiter=1)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        ctor(arg, quotechar=1)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        ctor(arg, lineterminator=None)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        ctor(arg, lineterminator=1)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        ctor(arg, quoting=None)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        ctor(arg, quoting=csv.QUOTE_ALL, quotechar='')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        ctor(arg, quoting=csv.QUOTE_ALL, quotechar=None)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        ctor(arg, quoting=csv.QUOTE_NONE, quotechar='')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    ctor(arg, delimiter=' ')
    ctor(arg, escapechar=' ')
    ctor(arg, quotechar=' ')
    ctor(arg, delimiter='\t', skipinitialspace=True)
    ctor(arg, escapechar='\t', skipinitialspace=True)
    ctor(arg, quotechar='\t', skipinitialspace=True)
    ctor(arg, delimiter=' ', skipinitialspace=True)
    ctor(arg, delimiter='^')
    ctor(arg, escapechar='^')
    ctor(arg, quotechar='^')
    ctor(arg, delimiter='\x85')
    ctor(arg, escapechar='\x85')
    ctor(arg, quotechar='\x85')
    ctor(arg, lineterminator='\x85')

def _test_default_attrs(ctor, *args):
    obj = ctor(*args)

    assert obj.dialect.delimiter == ','

    assert obj.dialect.doublequote is True

    assert obj.dialect.escapechar == None

    assert obj.dialect.lineterminator == '\r\n'

    assert obj.dialect.quotechar == '"'

    assert obj.dialect.quoting == csv.QUOTE_MINIMAL

    assert obj.dialect.skipinitialspace is False

    assert obj.dialect.strict is False

    try:
        delattr(obj.dialect, 'delimiter')
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass

    try:
        setattr(obj.dialect, 'delimiter', ':')
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass

    try:
        delattr(obj.dialect, 'quoting')
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass

    try:
        setattr(obj.dialect, 'quoting', None)
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass

def _test_dialect_attrs(ctor, *args):

    class dialect:
        delimiter = '-'
        doublequote = False
        escapechar = '^'
        lineterminator = '$'
        quotechar = '#'
        quoting = csv.QUOTE_ALL
        skipinitialspace = True
        strict = False
    args = args + (dialect,)
    obj = ctor(*args)

    assert obj.dialect.delimiter == '-'

    assert obj.dialect.doublequote is False

    assert obj.dialect.escapechar == '^'

    assert obj.dialect.lineterminator == '$'

    assert obj.dialect.quotechar == '#'

    assert obj.dialect.quoting == csv.QUOTE_ALL

    assert obj.dialect.skipinitialspace is True

    assert obj.dialect.strict is False

def _test_kw_attrs(ctor, *args):
    kwargs = dict(delimiter=':', doublequote=False, escapechar='\\', lineterminator='\r', quotechar='*', quoting=csv.QUOTE_NONE, skipinitialspace=True, strict=True)
    obj = ctor(*args, **kwargs)

    assert obj.dialect.delimiter == ':'

    assert obj.dialect.doublequote is False

    assert obj.dialect.escapechar == '\\'

    assert obj.dialect.lineterminator == '\r'

    assert obj.dialect.quotechar == '*'

    assert obj.dialect.quoting == csv.QUOTE_NONE

    assert obj.dialect.skipinitialspace is True

    assert obj.dialect.strict is True

def _write_error_test(exc, fields, **kwargs):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, **kwargs)
        try:
            writer.writerow(fields)
            raise AssertionError('expected exc')
        except exc:
            pass
        fileobj.seek(0)

        assert fileobj.read() == ''

def _write_test(fields, expect, **kwargs):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, **kwargs)
        writer.writerow(fields)
        fileobj.seek(0)

        assert fileobj.read() == expect + writer.dialect.lineterminator
_read_test(['no space, space,  spaces,\ttab'], [['no space', 'space', 'spaces', '\ttab']], skipinitialspace=True)
print("Test_Csv::test_read_skipinitialspace: ok")
