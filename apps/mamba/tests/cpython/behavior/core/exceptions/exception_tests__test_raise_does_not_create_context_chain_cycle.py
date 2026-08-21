# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_raise_does_not_create_context_chain_cycle"
# subject = "cpython.test_exceptions.ExceptionTests.test_raise_does_not_create_context_chain_cycle"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::test_raise_does_not_create_context_chain_cycle
"""Auto-ported test: ExceptionTests::test_raise_does_not_create_context_chain_cycle (CPython 3.12 oracle)."""


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
class A(Exception):
    pass

class B(Exception):
    pass

class C(Exception):
    pass
try:
    try:
        raise A
    except A as a_:
        a = a_
        try:
            raise B
        except B as b_:
            b = b_
            try:
                raise C
            except C as c_:
                c = c_

                assert isinstance(a, A)

                assert isinstance(b, B)

                assert isinstance(c, C)

                assert a.__context__ is None

                assert b.__context__ is a

                assert c.__context__ is b
                raise a
except A as e:
    exc = e

assert exc is a

assert a.__context__ is c

assert c.__context__ is b

assert b.__context__ is None
print("ExceptionTests::test_raise_does_not_create_context_chain_cycle: ok")
