# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_windows_error"
# subject = "cpython.test_exceptions.ExceptionTests.test_WindowsError"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::test_WindowsError
"""Auto-ported test: ExceptionTests::test_WindowsError (CPython 3.12 oracle)."""


import copy
import os
import sys
import unittest
import pickle
import weakref
import errno
from codecs import BOM_UTF8
from itertools import product
from textwrap import dedent
from test.support import captured_stderr, check_impl_detail, cpython_only, gc_collect, no_tracing, script_helper, SuppressCrashReport
from test.support.import_helper import import_module
from test.support.os_helper import TESTFN, unlink
from test.support.warnings_helper import check_warnings
from test import support


try:
    from _testcapi import INT_MAX
except ImportError:
    INT_MAX = 2 ** 31 - 1

class NaiveException(Exception):

    def __init__(self, x):
        self.x = x

class SlottedNaiveException(Exception):
    __slots__ = ('x',)

    def __init__(self, x):
        self.x = x

class BrokenStrException(Exception):

    def __str__(self):
        raise Exception('str() is broken')

def run_script(source):
    if isinstance(source, str):
        with open(TESTFN, 'w', encoding='utf-8') as testfile:
            testfile.write(dedent(source))
    else:
        with open(TESTFN, 'wb') as testfile:
            testfile.write(source)
    _rc, _out, err = script_helper.assert_python_failure('-Wd', '-X', 'utf8', TESTFN)
    return err.decode('utf-8').splitlines()


# --- test body ---
try:
    WindowsError
except NameError:
    pass
else:

    assert WindowsError is OSError

    assert str(OSError(1001)) == '1001'

    assert str(OSError(1001, 'message')) == '[Errno 1001] message'
    w = OSError(9, 'foo', 'bar')

    assert w.errno == 9

    assert w.winerror == None

    assert str(w) == "[Errno 9] foo: 'bar'"
    w = OSError(0, 'foo', 'bar', 3)

    assert w.errno == 2

    assert w.winerror == 3

    assert w.strerror == 'foo'

    assert w.filename == 'bar'

    assert w.filename2 == None

    assert str(w) == "[WinError 3] foo: 'bar'"
    w = OSError(0, 'foo', None, 1001)

    assert w.errno == 22

    assert w.winerror == 1001

    assert w.strerror == 'foo'

    assert w.filename == None

    assert w.filename2 == None

    assert str(w) == '[WinError 1001] foo'
    w = OSError('bar', 'foo')

    assert w.errno == 'bar'

    assert w.winerror == None

    assert w.strerror == 'foo'

    assert w.filename == None

    assert w.filename2 == None
print("ExceptionTests::test_WindowsError: ok")
