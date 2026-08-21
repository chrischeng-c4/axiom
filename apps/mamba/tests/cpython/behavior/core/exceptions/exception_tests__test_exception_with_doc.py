# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_exception_with_doc"
# subject = "cpython.test_exceptions.ExceptionTests.test_exception_with_doc"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::test_exception_with_doc
"""Auto-ported test: ExceptionTests::test_exception_with_doc (CPython 3.12 oracle)."""


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
import _testcapi
doc2 = 'This is a test docstring.'
doc4 = 'This is another test docstring.'

try:
    _testcapi.make_exception_with_doc('error1')
    raise AssertionError('expected SystemError')
except SystemError:
    pass
error1 = _testcapi.make_exception_with_doc('_testcapi.error1')

assert type(error1) is type

assert issubclass(error1, Exception)

assert error1.__doc__ is None
error2 = _testcapi.make_exception_with_doc('_testcapi.error2', doc2)

assert error2.__doc__ == doc2
error3 = _testcapi.make_exception_with_doc('_testcapi.error3', base=error2)

assert issubclass(error3, error2)

class C(object):
    pass
error4 = _testcapi.make_exception_with_doc('_testcapi.error4', doc4, (error3, C))

assert issubclass(error4, error3)

assert issubclass(error4, C)

assert error4.__doc__ == doc4
error5 = _testcapi.make_exception_with_doc('_testcapi.error5', '', error4, {'a': 1})

assert issubclass(error5, error4)

assert error5.a == 1

assert error5.__doc__ == ''
print("ExceptionTests::test_exception_with_doc: ok")
