# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_memory_error_subclasses"
# subject = "cpython.test_exceptions.ExceptionTests.test_memory_error_subclasses"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::test_memory_error_subclasses
"""Auto-ported test: ExceptionTests::test_memory_error_subclasses (CPython 3.12 oracle)."""


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
class TestException(MemoryError):
    pass
try:
    raise MemoryError
except MemoryError as exc:
    inst = exc
try:
    raise TestException
except Exception:
    pass
for _ in range(10):
    try:
        raise MemoryError
    except MemoryError as exc:
        pass
    gc_collect()
print("ExceptionTests::test_memory_error_subclasses: ok")
