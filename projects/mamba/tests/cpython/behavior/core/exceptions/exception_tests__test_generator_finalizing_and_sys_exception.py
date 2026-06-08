# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_generator_finalizing_and_sys_exception"
# subject = "cpython.test_exceptions.ExceptionTests.test_generator_finalizing_and_sys_exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::test_generator_finalizing_and_sys_exception
"""Auto-ported test: ExceptionTests::test_generator_finalizing_and_sys_exception (CPython 3.12 oracle)."""


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
def simple_gen():
    yield 1

def run_gen():
    gen = simple_gen()
    try:
        raise RuntimeError
    except RuntimeError:
        return next(gen)
run_gen()
gc_collect()

assert sys.exception() is None
print("ExceptionTests::test_generator_finalizing_and_sys_exception: ok")
