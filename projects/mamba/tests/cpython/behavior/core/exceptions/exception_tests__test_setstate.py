# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_setstate"
# subject = "cpython.test_exceptions.ExceptionTests.test_setstate"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::test_setstate
"""Auto-ported test: ExceptionTests::test_setstate (CPython 3.12 oracle)."""


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
e = Exception(42)
e.blah = 53

assert e.args == (42,)

assert e.blah == 53

try:
    getattr(e, 'a')
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass

try:
    getattr(e, 'b')
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
e.__setstate__({'a': 1, 'b': 2})

assert e.args == (42,)

assert e.blah == 53

assert e.a == 1

assert e.b == 2
e.__setstate__({'a': 11, 'args': (1, 2, 3), 'blah': 35})

assert e.args == (1, 2, 3)

assert e.blah == 35

assert e.a == 11

assert e.b == 2
print("ExceptionTests::test_setstate: ok")
