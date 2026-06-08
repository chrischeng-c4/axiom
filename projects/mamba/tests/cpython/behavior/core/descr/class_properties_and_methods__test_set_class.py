# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_set_class"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_set_class"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_set_class
"""Auto-ported test: ClassPropertiesAndMethods::test_set_class (CPython 3.12 oracle)."""


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

class D(object):
    pass

class E(object):
    pass

class F(D, E):
    pass
for cls in (C, D, E, F):
    for cls2 in (C, D, E, F):
        x = cls()
        x.__class__ = cls2

        assert x.__class__ is cls2
        x.__class__ = cls

        assert x.__class__ is cls

def cant(x, C):
    try:
        x.__class__ = C
    except TypeError:
        pass
    else:
        self.fail("shouldn't allow %r.__class__ = %r" % (x, C))
    try:
        delattr(x, '__class__')
    except (TypeError, AttributeError):
        pass
    else:
        self.fail("shouldn't allow del %r.__class__" % x)
cant(C(), list)
cant(list(), C)
cant(C(), 1)
cant(C(), object)
cant(object(), list)
cant(list(), object)

class Int(int):
    __slots__ = []
cant(True, int)
cant(2, bool)
o = object()
cant(o, int)
cant(o, type(None))
del o

class G(object):
    __slots__ = ['a', 'b']

class H(object):
    __slots__ = ['b', 'a']

class I(object):
    __slots__ = ['a', 'b']

class J(object):
    __slots__ = ['c', 'b']

class K(object):
    __slots__ = ['a', 'b', 'd']

class L(H):
    __slots__ = ['e']

class M(I):
    __slots__ = ['e']

class N(J):
    __slots__ = ['__weakref__']

class P(J):
    __slots__ = ['__dict__']

class Q(J):
    pass

class R(J):
    __slots__ = ['__dict__', '__weakref__']
for cls, cls2 in ((G, H), (G, I), (I, H), (Q, R), (R, Q)):
    x = cls()
    x.a = 1
    x.__class__ = cls2

    assert x.__class__ is cls2

    assert x.a == 1
    x.__class__ = cls

    assert x.__class__ is cls

    assert x.a == 1
for cls in (G, J, K, L, M, N, P, R, list, Int):
    for cls2 in (G, J, K, L, M, N, P, R, list, Int):
        if cls is cls2:
            continue
        cant(cls(), cls2)

class O(object):
    pass

class A(object):

    def __del__(self):
        self.__class__ = O
l = [A() for x in range(100)]
del l
print("ClassPropertiesAndMethods::test_set_class: ok")
