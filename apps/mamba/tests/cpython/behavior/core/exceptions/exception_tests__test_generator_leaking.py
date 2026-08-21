# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_generator_leaking"
# subject = "cpython.test_exceptions.ExceptionTests.test_generator_leaking"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::test_generator_leaking
"""Auto-ported test: ExceptionTests::test_generator_leaking (CPython 3.12 oracle)."""


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
def yield_raise():
    try:
        raise KeyError('caught')
    except KeyError:
        yield sys.exception()
        yield sys.exception()
    yield sys.exception()
g = yield_raise()

assert isinstance(next(g), KeyError)

assert sys.exception() is None

assert isinstance(next(g), KeyError)

assert sys.exception() is None

assert next(g) is None
try:
    raise TypeError('foo')
except TypeError:
    g = yield_raise()

    assert isinstance(next(g), KeyError)

    assert isinstance(sys.exception(), TypeError)

    assert isinstance(next(g), KeyError)

    assert isinstance(sys.exception(), TypeError)

    assert isinstance(next(g), TypeError)
    del g

    assert isinstance(sys.exception(), TypeError)
print("ExceptionTests::test_generator_leaking: ok")
