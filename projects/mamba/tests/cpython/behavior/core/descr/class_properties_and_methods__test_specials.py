# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_specials"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_specials"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_specials
"""Auto-ported test: ClassPropertiesAndMethods::test_specials (CPython 3.12 oracle)."""


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

    def __getitem__(self, i):
        if 0 <= i < 10:
            return i
        raise IndexError
c1 = C()
c2 = C()

assert not not c1

assert id(c1) != id(c2)
hash(c1)
hash(c2)

assert c1 == c1

assert c1 != c2

assert not c1 != c1

assert not c1 == c2

assert str(c1).find('C object at ') >= 0

assert str(c1) == repr(c1)

assert -1 not in c1
for i in range(10):

    assert i in c1

assert 10 not in c1

class D(object):

    def __getitem__(self, i):
        if 0 <= i < 10:
            return i
        raise IndexError
d1 = D()
d2 = D()

assert not not d1

assert id(d1) != id(d2)
hash(d1)
hash(d2)

assert d1 == d1

assert d1 != d2

assert not d1 != d1

assert not d1 == d2

assert str(d1).find('D object at ') >= 0

assert str(d1) == repr(d1)

assert -1 not in d1
for i in range(10):

    assert i in d1

assert 10 not in d1

class Proxy(object):

    def __init__(self, x):
        self.x = x

    def __bool__(self):
        return not not self.x

    def __hash__(self):
        return hash(self.x)

    def __eq__(self, other):
        return self.x == other

    def __ne__(self, other):
        return self.x != other

    def __ge__(self, other):
        return self.x >= other

    def __gt__(self, other):
        return self.x > other

    def __le__(self, other):
        return self.x <= other

    def __lt__(self, other):
        return self.x < other

    def __str__(self):
        return 'Proxy:%s' % self.x

    def __repr__(self):
        return 'Proxy(%r)' % self.x

    def __contains__(self, value):
        return value in self.x
p0 = Proxy(0)
p1 = Proxy(1)
p_1 = Proxy(-1)

assert not p0

assert not not p1

assert hash(p0) == hash(0)

assert p0 == p0

assert p0 != p1

assert not p0 != p0

assert (not p0) == p1

assert p0 < p1

assert p0 <= p1

assert p1 > p0

assert p1 >= p0

assert str(p0) == 'Proxy:0'

assert repr(p0) == 'Proxy(0)'
p10 = Proxy(range(10))

assert -1 not in p10
for i in range(10):

    assert i in p10

assert 10 not in p10
print("ClassPropertiesAndMethods::test_specials: ok")
