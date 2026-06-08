# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_subclass_propagation"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_subclass_propagation"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_subclass_propagation
"""Auto-ported test: ClassPropertiesAndMethods::test_subclass_propagation (CPython 3.12 oracle)."""


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
    pass

class B(A):
    pass

class C(A):
    pass

class D(B, C):
    pass
d = D()
orig_hash = hash(d)
A.__hash__ = lambda self: 42

assert hash(d) == 42
C.__hash__ = lambda self: 314

assert hash(d) == 314
B.__hash__ = lambda self: 144

assert hash(d) == 144
D.__hash__ = lambda self: 100

assert hash(d) == 100
D.__hash__ = None

try:
    hash(d)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
del D.__hash__

assert hash(d) == 144
B.__hash__ = None

try:
    hash(d)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
del B.__hash__

assert hash(d) == 314
C.__hash__ = None

try:
    hash(d)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
del C.__hash__

assert hash(d) == 42
A.__hash__ = None

try:
    hash(d)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
del A.__hash__

assert hash(d) == orig_hash
d.foo = 42
d.bar = 42

assert d.foo == 42

assert d.bar == 42

def __getattribute__(self, name):
    if name == 'foo':
        return 24
    return object.__getattribute__(self, name)
A.__getattribute__ = __getattribute__

assert d.foo == 24

assert d.bar == 42

def __getattr__(self, name):
    if name in ('spam', 'foo', 'bar'):
        return 'hello'
    raise AttributeError(name)
B.__getattr__ = __getattr__

assert d.spam == 'hello'

assert d.foo == 24

assert d.bar == 42
del A.__getattribute__

assert d.foo == 42
del d.foo

assert d.foo == 'hello'

assert d.bar == 42
del B.__getattr__
try:
    d.foo
except AttributeError:
    pass
else:

    raise AssertionError('d.foo should be undefined now')

class A(object):
    pass

class B(A):
    pass
del B
support.gc_collect()
A.__setitem__ = lambda *a: None
print("ClassPropertiesAndMethods::test_subclass_propagation: ok")
