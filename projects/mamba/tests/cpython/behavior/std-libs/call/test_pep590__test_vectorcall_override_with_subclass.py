# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "call"
# dimension = "behavior"
# case = "test_pep590__test_vectorcall_override_with_subclass"
# subject = "cpython.test_call.TestPEP590.test_vectorcall_override_with_subclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_call.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_call.py::TestPEP590::test_vectorcall_override_with_subclass
"""Auto-ported test: TestPEP590::test_vectorcall_override_with_subclass (CPython 3.12 oracle)."""


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
"""Setting __call__ on a superclass should disable vectorcall"""
SuperType = _testcapi.make_vectorcall_class()

class DerivedType(SuperType):
    pass
instance = DerivedType()
UnaffectedType1 = _testcapi.make_vectorcall_class(DerivedType)
UnaffectedType2 = _testcapi.make_vectorcall_class(SuperType)

assert issubclass(UnaffectedType1, DerivedType)

assert issubclass(UnaffectedType2, SuperType)

assert instance() == 'tp_call'

assert _testcapi.has_vectorcall_flag(SuperType) == True

assert _testcapi.has_vectorcall_flag(DerivedType) == True

assert _testcapi.has_vectorcall_flag(UnaffectedType1) == True

assert _testcapi.has_vectorcall_flag(UnaffectedType2) == True
instance.set_vectorcall(SuperType)

assert instance() == 'vectorcall'

assert _testcapi.has_vectorcall_flag(SuperType) == True

assert _testcapi.has_vectorcall_flag(DerivedType) == True

assert _testcapi.has_vectorcall_flag(UnaffectedType1) == True

assert _testcapi.has_vectorcall_flag(UnaffectedType2) == True
SuperType.__call__ = lambda self: 'custom'

assert instance() == 'custom'

assert _testcapi.has_vectorcall_flag(SuperType) == False

assert _testcapi.has_vectorcall_flag(DerivedType) == False

assert _testcapi.has_vectorcall_flag(UnaffectedType1) == True

assert _testcapi.has_vectorcall_flag(UnaffectedType2) == True
print("TestPEP590::test_vectorcall_override_with_subclass: ok")
