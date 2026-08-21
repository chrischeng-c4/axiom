# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_supers"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_supers"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_supers
"""Auto-ported test: ClassPropertiesAndMethods::test_supers (CPython 3.12 oracle)."""


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

    def meth(self, a):
        return 'A(%r)' % a

assert A().meth(1) == 'A(1)'

class B(A):

    def __init__(self):
        self.__super = super(B, self)

    def meth(self, a):
        return 'B(%r)' % a + self.__super.meth(a)

assert B().meth(2) == 'B(2)A(2)'

class C(A):

    def meth(self, a):
        return 'C(%r)' % a + self.__super.meth(a)
C._C__super = super(C)

assert C().meth(3) == 'C(3)A(3)'

class D(C, B):

    def meth(self, a):
        return 'D(%r)' % a + super(D, self).meth(a)

assert D().meth(4) == 'D(4)C(4)B(4)A(4)'

class mysuper(super):

    def __init__(self, *args):
        return super(mysuper, self).__init__(*args)

class E(D):

    def meth(self, a):
        return 'E(%r)' % a + mysuper(E, self).meth(a)

assert E().meth(5) == 'E(5)D(5)C(5)B(5)A(5)'

class F(E):

    def meth(self, a):
        s = self.__super
        return 'F(%r)[%s]' % (a, s.__class__.__name__) + s.meth(a)
F._F__super = mysuper(F)

assert F().meth(6) == 'F(6)[mysuper]E(6)D(6)C(6)B(6)A(6)'
try:
    super(D, 42)
except TypeError:
    pass
else:

    raise AssertionError("shouldn't allow super(D, 42)")
try:
    super(D, C())
except TypeError:
    pass
else:

    raise AssertionError("shouldn't allow super(D, C())")
try:
    super(D).__get__(12)
except TypeError:
    pass
else:

    raise AssertionError("shouldn't allow super(D).__get__(12)")
try:
    super(D).__get__(C())
except TypeError:
    pass
else:

    raise AssertionError("shouldn't allow super(D).__get__(C())")

class DDbase(object):

    def getx(self):
        return 42
    x = property(getx)

class DDsub(DDbase):

    def getx(self):
        return 'hello'
    x = property(getx)
dd = DDsub()

assert dd.x == 'hello'

assert super(DDsub, dd).x == 42

class Base(object):
    aProp = property(lambda self: 'foo')

class Sub(Base):

    @classmethod
    def test(klass):
        return super(Sub, klass).aProp

assert Sub.test() == Base.aProp
try:
    super(Base, kw=1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ClassPropertiesAndMethods::test_supers: ok")
