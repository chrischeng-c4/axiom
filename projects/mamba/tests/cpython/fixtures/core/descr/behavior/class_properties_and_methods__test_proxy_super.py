# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_proxy_super"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_proxy_super"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_proxy_super
"""Auto-ported test: ClassPropertiesAndMethods::test_proxy_super (CPython 3.12 oracle)."""


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

class B(object):

    def f(self):
        return 'B.f'

class C(B):

    def f(self):
        return super(C, self).f() + '->C.f'
obj = C()
p = Proxy(obj)

assert C.__dict__['f'](p) == 'B.f->C.f'
print("ClassPropertiesAndMethods::test_proxy_super: ok")
