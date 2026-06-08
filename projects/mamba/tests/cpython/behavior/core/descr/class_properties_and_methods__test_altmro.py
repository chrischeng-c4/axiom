# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_altmro"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_altmro"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_altmro
"""Auto-ported test: ClassPropertiesAndMethods::test_altmro (CPython 3.12 oracle)."""


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
class A(object):

    def f(self):
        return 'A'

class B(A):
    pass

class C(A):

    def f(self):
        return 'C'

class D(B, C):
    pass

assert A.mro() == [A, object]

assert A.__mro__ == (A, object)

assert B.mro() == [B, A, object]

assert B.__mro__ == (B, A, object)

assert C.mro() == [C, A, object]

assert C.__mro__ == (C, A, object)

assert D.mro() == [D, B, C, A, object]

assert D.__mro__ == (D, B, C, A, object)

assert D().f() == 'C'

class PerverseMetaType(type):

    def mro(cls):
        L = type.mro(cls)
        L.reverse()
        return L

class X(D, B, C, A, metaclass=PerverseMetaType):
    pass

assert X.__mro__ == (object, A, C, B, D, X)

assert X().f() == 'A'
try:

    class _metaclass(type):

        def mro(self):
            return [self, dict, object]

    class X(object, metaclass=_metaclass):
        pass
    x = object.__new__(X)
    x[5] = 6
except TypeError:
    pass
else:

    raise AssertionError('devious mro() return not caught')
try:

    class _metaclass(type):

        def mro(self):
            return [1]

    class X(object, metaclass=_metaclass):
        pass
except TypeError:
    pass
else:

    raise AssertionError('non-class mro() return not caught')
try:

    class _metaclass(type):

        def mro(self):
            return 1

    class X(object, metaclass=_metaclass):
        pass
except TypeError:
    pass
else:

    raise AssertionError('non-sequence mro() return not caught')
print("ClassPropertiesAndMethods::test_altmro: ok")
