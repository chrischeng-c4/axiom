# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "call"
# dimension = "behavior"
# case = "fast_call_tests__test_vectorcall"
# subject = "cpython.test_call.FastCallTests.test_vectorcall"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_call.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_call.py::FastCallTests::test_vectorcall
"""Auto-ported test: FastCallTests::test_vectorcall (CPython 3.12 oracle)."""


import unittest
from test.support import cpython_only, requires_limited_api, skip_on_s390x, is_wasi, Py_DEBUG
import struct
import collections
import itertools
import gc
import contextlib
import sys
import types


try:
    import _testcapi
except ImportError:
    _testcapi = None

class BadStr(str):

    def __eq__(self, other):
        return True

    def __hash__(self):
        return str.__hash__(self) ^ 3

def pyfunc(arg1, arg2):
    return [arg1, arg2]

def pyfunc_noarg():
    return 'noarg'

class PythonClass:

    def method(self, arg1, arg2):
        return [arg1, arg2]

    def method_noarg(self):
        return 'noarg'

    @classmethod
    def class_method(cls):
        return 'classmethod'

    @staticmethod
    def static_method():
        return 'staticmethod'

PYTHON_INSTANCE = PythonClass()

NULL_OR_EMPTY = object()

Py_TPFLAGS_HAVE_VECTORCALL = 1 << 11

Py_TPFLAGS_METHOD_DESCRIPTOR = 1 << 17

def testfunction(self):
    """some doc"""
    return self

def testfunction_kw(self, *, kw):
    """some doc"""
    return self

ADAPTIVE_WARMUP_DELAY = 2

class A:

    def method_two_args(self, x, y):
        pass

    @staticmethod
    def static_no_args():
        pass

    @staticmethod
    def positional_only(arg, /):
        pass


# --- test body ---
CALLS_POSARGS = [(pyfunc, (1, 2), [1, 2]), (pyfunc_noarg, (), 'noarg'), (PythonClass.class_method, (), 'classmethod'), (PythonClass.static_method, (), 'staticmethod'), (PYTHON_INSTANCE.method, (1, 2), [1, 2]), (PYTHON_INSTANCE.method_noarg, (), 'noarg'), (PYTHON_INSTANCE.class_method, (), 'classmethod'), (PYTHON_INSTANCE.static_method, (), 'staticmethod')]
CALLS_KWARGS = [(pyfunc, (1,), {'arg2': 2}, [1, 2]), (pyfunc, (), {'arg1': 1, 'arg2': 2}, [1, 2]), (PYTHON_INSTANCE.method, (1,), {'arg2': 2}, [1, 2]), (PYTHON_INSTANCE.method, (), {'arg1': 1, 'arg2': 2}, [1, 2])]

def check_result(result, expected):
    if isinstance(expected, tuple) and expected[-1] is NULL_OR_EMPTY:
        if result[-1] in ({}, None):
            expected = (*expected[:-1], result[-1])

    assert result == expected
for func, args, expected in CALLS_POSARGS:
    result = _testcapi.pyobject_vectorcall(func, args, None)
    check_result(result, expected)
    result = _testcapi.pyobject_vectorcall(func, args, ())
    check_result(result, expected)
    if not args:
        result = _testcapi.pyobject_vectorcall(func, None, None)
        check_result(result, expected)
        result = _testcapi.pyobject_vectorcall(func, None, ())
        check_result(result, expected)
for func, args, kwargs, expected in CALLS_KWARGS:
    kwnames = tuple(kwargs.keys())
    args = args + tuple(kwargs.values())
    result = _testcapi.pyobject_vectorcall(func, args, kwnames)
    check_result(result, expected)
print("FastCallTests::test_vectorcall: ok")
