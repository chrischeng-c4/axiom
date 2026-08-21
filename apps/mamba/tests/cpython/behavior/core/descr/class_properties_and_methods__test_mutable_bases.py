# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_mutable_bases"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_mutable_bases"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_mutable_bases
"""Auto-ported test: ClassPropertiesAndMethods::test_mutable_bases (CPython 3.12 oracle)."""


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
class C(object):
    pass

class C2(object):

    def __getattribute__(self, attr):
        if attr == 'a':
            return 2
        else:
            return super(C2, self).__getattribute__(attr)

    def meth(self):
        return 1

class D(C):
    pass

class E(D):
    pass
d = D()
e = E()
D.__bases__ = (C,)
D.__bases__ = (C2,)

assert d.meth() == 1

assert e.meth() == 1

assert d.a == 2

assert e.a == 2

assert C2.__subclasses__() == [D]
try:
    del D.__bases__
except (TypeError, AttributeError):
    pass
else:

    raise AssertionError("shouldn't be able to delete .__bases__")
try:
    D.__bases__ = ()
except TypeError as msg:
    if str(msg) == "a new-style class can't have only classic bases":

        raise AssertionError('wrong error message for .__bases__ = ()')
else:

    raise AssertionError("shouldn't be able to set .__bases__ to ()")
try:
    D.__bases__ = (D,)
except TypeError:
    pass
else:

    raise AssertionError("shouldn't be able to create inheritance cycles")
try:
    D.__bases__ = (C, C)
except TypeError:
    pass
else:

    raise AssertionError("didn't detect repeated base classes")
try:
    D.__bases__ = (E,)
except TypeError:
    pass
else:

    raise AssertionError("shouldn't be able to create inheritance cycles")
print("ClassPropertiesAndMethods::test_mutable_bases: ok")
