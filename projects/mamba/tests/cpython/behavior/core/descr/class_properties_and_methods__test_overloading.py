# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_overloading"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_overloading"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_overloading
"""Auto-ported test: ClassPropertiesAndMethods::test_overloading (CPython 3.12 oracle)."""


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
class B(object):
    """Intermediate class because object doesn't have a __setattr__"""

class C(B):

    def __getattr__(self, name):
        if name == 'foo':
            return ('getattr', name)
        else:
            raise AttributeError

    def __setattr__(self, name, value):
        if name == 'foo':
            self.setattr = (name, value)
        else:
            return B.__setattr__(self, name, value)

    def __delattr__(self, name):
        if name == 'foo':
            self.delattr = name
        else:
            return B.__delattr__(self, name)

    def __getitem__(self, key):
        return ('getitem', key)

    def __setitem__(self, key, value):
        self.setitem = (key, value)

    def __delitem__(self, key):
        self.delitem = key
a = C()

assert a.foo == ('getattr', 'foo')
a.foo = 12

assert a.setattr == ('foo', 12)
del a.foo

assert a.delattr == 'foo'

assert a[12] == ('getitem', 12)
a[12] = 21

assert a.setitem == (12, 21)
del a[12]

assert a.delitem == 12

assert a[0:10] == ('getitem', slice(0, 10))
a[0:10] = 'foo'

assert a.setitem == (slice(0, 10), 'foo')
del a[0:10]

assert a.delitem == slice(0, 10)
print("ClassPropertiesAndMethods::test_overloading: ok")
