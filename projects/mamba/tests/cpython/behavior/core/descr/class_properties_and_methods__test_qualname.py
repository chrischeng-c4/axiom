# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_qualname"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_qualname"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_qualname
"""Auto-ported test: ClassPropertiesAndMethods::test_qualname (CPython 3.12 oracle)."""


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
descriptors = [str.lower, complex.real, float.real, int.__add__]
types = ['method', 'member', 'getset', 'wrapper']
for d, n in zip(descriptors, types):

    assert type(d).__name__ == n + '_descriptor'
for d in descriptors:
    qualname = d.__objclass__.__qualname__ + '.' + d.__name__

    assert d.__qualname__ == qualname

assert str.lower.__qualname__ == 'str.lower'

assert complex.real.__qualname__ == 'complex.real'

assert float.real.__qualname__ == 'float.real'

assert int.__add__.__qualname__ == 'int.__add__'

class X:
    pass
try:
    del X.__qualname__
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    type.__dict__['__qualname__'].__set__(str, 'Oink')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
global Y

class Y:

    class Inside:
        pass

assert Y.__qualname__ == 'Y'

assert Y.Inside.__qualname__ == 'Y.Inside'
print("ClassPropertiesAndMethods::test_qualname: ok")
