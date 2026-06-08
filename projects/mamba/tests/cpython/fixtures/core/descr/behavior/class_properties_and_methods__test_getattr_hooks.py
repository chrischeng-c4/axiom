# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_getattr_hooks"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_getattr_hooks"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_getattr_hooks
"""Auto-ported test: ClassPropertiesAndMethods::test_getattr_hooks (CPython 3.12 oracle)."""


import builtins
import copyreg
import gc
import itertools
import math
import pickle
import random
import string
import sys
import types
import unittest
import warnings
import weakref
from copy import deepcopy
from contextlib import redirect_stdout
from test import support
from test.support.testcase import ExtraAssertions


try:
    import _testcapi
except ImportError:
    _testcapi = None

try:
    import xxsubtype
except ImportError:
    xxsubtype = None

class DebugHelperMeta(type):
    """
    Sets default __doc__ and simplifies repr() output.
    """

    def __new__(mcls, name, bases, attrs):
        if attrs.get('__doc__') is None:
            attrs['__doc__'] = name
        return type.__new__(mcls, name, bases, attrs)

    def __repr__(cls):
        return repr(cls.__name__)


# --- test body ---
class Descriptor(object):
    counter = 0

    def __get__(self, obj, objtype=None):

        def getter(name):
            self.counter += 1
            raise AttributeError(name)
        return getter
descr = Descriptor()

class A(object):
    __getattribute__ = descr

class B(object):
    __getattr__ = descr

class C(object):
    __getattribute__ = descr
    __getattr__ = descr

try:
    getattr(A(), 'attr')
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass

assert descr.counter == 1

try:
    getattr(B(), 'attr')
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass

assert descr.counter == 2

try:
    getattr(C(), 'attr')
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass

assert descr.counter == 4

class EvilGetattribute(object):

    def __getattr__(self, name):
        raise AttributeError(name)

    def __getattribute__(self, name):
        del EvilGetattribute.__getattr__
        for i in range(5):
            gc.collect()
        raise AttributeError(name)

try:
    getattr(EvilGetattribute(), 'attr')
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
print("ClassPropertiesAndMethods::test_getattr_hooks: ok")
