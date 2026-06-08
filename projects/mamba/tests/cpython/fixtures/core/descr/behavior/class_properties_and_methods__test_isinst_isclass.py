# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_isinst_isclass"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_isinst_isclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_isinst_isclass
"""Auto-ported test: ClassPropertiesAndMethods::test_isinst_isclass (CPython 3.12 oracle)."""


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
class Proxy(object):

    def __init__(self, obj):
        self.__obj = obj

    def __getattribute__(self, name):
        if name.startswith('_Proxy__'):
            return object.__getattribute__(self, name)
        else:
            return getattr(self.__obj, name)

class C:
    pass
a = C()
pa = Proxy(a)

assert isinstance(a, C)

assert isinstance(pa, C)

class D(C):
    pass
a = D()
pa = Proxy(a)

assert isinstance(a, C)

assert isinstance(pa, C)

class C(object):
    pass
a = C()
pa = Proxy(a)

assert isinstance(a, C)

assert isinstance(pa, C)

class D(C):
    pass
a = D()
pa = Proxy(a)

assert isinstance(a, C)

assert isinstance(pa, C)
print("ClassPropertiesAndMethods::test_isinst_isclass: ok")
