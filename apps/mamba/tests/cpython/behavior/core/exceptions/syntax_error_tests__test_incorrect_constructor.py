# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "syntax_error_tests__test_incorrect_constructor"
# subject = "cpython.test_exceptions.SyntaxErrorTests.test_incorrect_constructor"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exceptions.py::SyntaxErrorTests::test_incorrect_constructor
"""Auto-ported test: SyntaxErrorTests::test_incorrect_constructor (CPython 3.12 oracle)."""


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
maxDiff = None
args = ('bad.py', 1, 2)

try:
    SyntaxError('bad bad', args)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
args = ('bad.py', 1, 2, 4, 5, 6, 7)

try:
    SyntaxError('bad bad', args)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
args = ('bad.py', 1, 2, 'abcdefg', 1)

try:
    SyntaxError('bad bad', args)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("SyntaxErrorTests::test_incorrect_constructor: ok")
