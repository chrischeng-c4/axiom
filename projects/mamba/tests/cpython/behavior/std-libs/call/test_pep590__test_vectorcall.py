# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "call"
# dimension = "behavior"
# case = "test_pep590__test_vectorcall"
# subject = "cpython.test_call.TestPEP590.test_vectorcall"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_call.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_call.py::TestPEP590::test_vectorcall
"""Auto-ported test: TestPEP590::test_vectorcall (CPython 3.12 oracle)."""


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
calls = [(len, (range(42),), {}, 42), (list.append, ([], 0), {}, None), ([].append, (0,), {}, None), (sum, ([36],), {'start': 6}, 42), (testfunction, (42,), {}, 42), (testfunction_kw, (42,), {'kw': None}, 42), (_testcapi.MethodDescriptorBase(), (0,), {}, True), (_testcapi.MethodDescriptorDerived(), (0,), {}, True), (_testcapi.MethodDescriptor2(), (0,), {}, False)]
from _testcapi import pyobject_vectorcall, pyvectorcall_call
from types import MethodType
from functools import partial

def vectorcall(func, args, kwargs):
    args = (*args, *kwargs.values())
    kwnames = tuple(kwargs)
    return pyobject_vectorcall(func, args, kwnames)
for func, args, kwargs, expected in calls:
    if not kwargs:

        assert expected == pyvectorcall_call(func, args)

    assert expected == pyvectorcall_call(func, args, kwargs)

class MethodDescriptorHeap(_testcapi.MethodDescriptorBase):
    pass

class MethodDescriptorOverridden(_testcapi.MethodDescriptorBase):

    def __call__(self, n):
        return 'new'

class SuperBase:

    def __call__(self, *args):
        return super().__call__(*args)

class MethodDescriptorSuper(SuperBase, _testcapi.MethodDescriptorBase):

    def __call__(self, *args):
        return super().__call__(*args)
calls += [(dict.update, ({},), {'key': True}, None), ({}.update, ({},), {'key': True}, None), (MethodDescriptorHeap(), (0,), {}, True), (MethodDescriptorOverridden(), (0,), {}, 'new'), (MethodDescriptorSuper(), (0,), {}, True)]
for func, args, kwargs, expected in calls:
    args1 = args[1:]
    meth = MethodType(func, args[0])
    wrapped = partial(func)
    if not kwargs:

        assert expected == func(*args)

        assert expected == pyobject_vectorcall(func, args, None)

        assert expected == meth(*args1)

        assert expected == wrapped(*args)

    assert expected == func(*args, **kwargs)

    assert expected == vectorcall(func, args, kwargs)

    assert expected == meth(*args1, **kwargs)

    assert expected == wrapped(*args, **kwargs)
print("TestPEP590::test_vectorcall: ok")
