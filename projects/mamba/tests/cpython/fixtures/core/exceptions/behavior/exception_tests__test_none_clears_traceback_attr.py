# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_none_clears_traceback_attr"
# subject = "cpython.test_exceptions.ExceptionTests.testNoneClearsTracebackAttr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::testNoneClearsTracebackAttr
"""Auto-ported test: ExceptionTests::testNoneClearsTracebackAttr (CPython 3.12 oracle)."""


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
def _check_generator_cleanup_exc_state(testfunc):

    class MyException(Exception):

        def __init__(self, obj):
            self.obj = obj

    class MyObj:
        pass

    def raising_gen():
        try:
            raise MyException(obj)
        except MyException:
            yield
    obj = MyObj()
    wr = weakref.ref(obj)
    g = raising_gen()
    next(g)
    testfunc(g)
    g = obj = None
    gc_collect()
    obj = wr()

    assert obj is None

def check(src, lineno, offset, end_lineno=None, end_offset=None, encoding='utf-8'):
    try:
        compile(src, '<fragment>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)

    assert cm.exception.lineno == lineno

    assert cm.exception.offset == offset
    if end_lineno is not None:

        assert cm.exception.end_lineno == end_lineno
    if end_offset is not None:

        assert cm.exception.end_offset == end_offset
    if cm.exception.text is not None:
        if not isinstance(src, str):
            src = src.decode(encoding, 'replace')
        line = src.split('\n')[lineno - 1]

        assert line in cm.exception.text

def raise_catch(exc, excname):
    try:
        raise exc('spam')
    except exc as err:
        buf1 = str(err)
    try:
        raise exc('spam')
    except exc as err:
        buf2 = str(err)

    assert buf1 == buf2

    assert exc.__name__ == excname
try:
    raise IndexError(4)
except Exception as e:
    tb = e.__traceback__
e = Exception()
e.__traceback__ = tb
e.__traceback__ = None

assert e.__traceback__ == None
print("ExceptionTests::testNoneClearsTracebackAttr: ok")
