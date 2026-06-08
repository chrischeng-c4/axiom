# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_no_hang_on_context_chain_cycle3"
# subject = "cpython.test_exceptions.ExceptionTests.test_no_hang_on_context_chain_cycle3"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::test_no_hang_on_context_chain_cycle3
"""Auto-ported test: ExceptionTests::test_no_hang_on_context_chain_cycle3 (CPython 3.12 oracle)."""


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

class D(Exception):
    pass

class E(Exception):
    pass
try:
    try:
        raise A()
    except A as _a:
        a = _a
        try:
            raise B()
        except B as _b:
            b = _b
            try:
                raise C()
            except C as _c:
                c = _c
                a.__context__ = c
                try:
                    raise D()
                except D as _d:
                    d = _d
                    e = E()
                    raise e
    raise AssertionError('expected E')
except E as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert cm.exception is e

assert e.__context__ is d

assert d.__context__ is c

assert c.__context__ is b

assert b.__context__ is a

assert a.__context__ is c
print("ExceptionTests::test_no_hang_on_context_chain_cycle3: ok")
