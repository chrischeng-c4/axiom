# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_unicode_change_attributes"
# subject = "cpython.test_exceptions.ExceptionTests.test_unicode_change_attributes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::test_unicode_change_attributes
"""Auto-ported test: ExceptionTests::test_unicode_change_attributes (CPython 3.12 oracle)."""


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
u = UnicodeEncodeError('baz', 'xxxxx', 1, 5, 'foo')

assert str(u) == "'baz' codec can't encode characters in position 1-4: foo"
u.end = 2

assert str(u) == "'baz' codec can't encode character '\\x78' in position 1: foo"
u.end = 5
u.reason = 965230951443685724997

assert str(u) == "'baz' codec can't encode characters in position 1-4: 965230951443685724997"
u.encoding = 4000

assert str(u) == "'4000' codec can't encode characters in position 1-4: 965230951443685724997"
u.start = 1000

assert str(u) == "'4000' codec can't encode characters in position 1000-4: 965230951443685724997"
u = UnicodeDecodeError('baz', b'xxxxx', 1, 5, 'foo')

assert str(u) == "'baz' codec can't decode bytes in position 1-4: foo"
u.end = 2

assert str(u) == "'baz' codec can't decode byte 0x78 in position 1: foo"
u.end = 5
u.reason = 965230951443685724997

assert str(u) == "'baz' codec can't decode bytes in position 1-4: 965230951443685724997"
u.encoding = 4000

assert str(u) == "'4000' codec can't decode bytes in position 1-4: 965230951443685724997"
u.start = 1000

assert str(u) == "'4000' codec can't decode bytes in position 1000-4: 965230951443685724997"
u = UnicodeTranslateError('xxxx', 1, 5, 'foo')

assert str(u) == "can't translate characters in position 1-4: foo"
u.end = 2

assert str(u) == "can't translate character '\\x78' in position 1: foo"
u.end = 5
u.reason = 965230951443685724997

assert str(u) == "can't translate characters in position 1-4: 965230951443685724997"
u.start = 1000

assert str(u) == "can't translate characters in position 1000-4: 965230951443685724997"
print("ExceptionTests::test_unicode_change_attributes: ok")
