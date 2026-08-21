# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "class_properties_and_methods__test_basic_inheritance"
# subject = "cpython.test_descr.ClassPropertiesAndMethods.test_basic_inheritance"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_descr.py::ClassPropertiesAndMethods::test_basic_inheritance
"""Auto-ported test: ClassPropertiesAndMethods::test_basic_inheritance (CPython 3.12 oracle)."""


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
class hexint(int):

    def __repr__(self):
        return hex(self)

    def __add__(self, other):
        return hexint(int.__add__(self, other))

assert repr(hexint(7) + 9) == '0x10'

assert repr(hexint(1000) + 7) == '0x3ef'
a = hexint(12345)

assert a == 12345

assert int(a) == 12345

assert int(a).__class__ is int

assert hash(a) == hash(12345)

assert (+a).__class__ is int

assert (a >> 0).__class__ is int

assert (a << 0).__class__ is int

assert (hexint(0) << 12).__class__ is int

assert (hexint(0) >> 12).__class__ is int

class octlong(int):
    __slots__ = []

    def __str__(self):
        return oct(self)

    def __add__(self, other):
        return self.__class__(super(octlong, self).__add__(other))
    __radd__ = __add__

assert str(octlong(3) + 5) == '0o10'

assert str(5 + octlong(3000)) == '0o5675'
a = octlong(12345)

assert a == 12345

assert int(a) == 12345

assert hash(a) == hash(12345)

assert int(a).__class__ is int

assert (+a).__class__ is int

assert (-a).__class__ is int

assert (-octlong(0)).__class__ is int

assert (a >> 0).__class__ is int

assert (a << 0).__class__ is int

assert (a - 0).__class__ is int

assert (a * 1).__class__ is int

assert (a ** 1).__class__ is int

assert (a // 1).__class__ is int

assert (1 * a).__class__ is int

assert (a | 0).__class__ is int

assert (a ^ 0).__class__ is int

assert (a & -1).__class__ is int

assert (octlong(0) << 12).__class__ is int

assert (octlong(0) >> 12).__class__ is int

assert abs(octlong(0)).__class__ is int

class longclone(int):
    pass
a = longclone(1)

assert (a + 0).__class__ is int

assert (0 + a).__class__ is int
a = longclone(-1)

assert a.__dict__ == {}

assert int(a) == -1

class precfloat(float):
    __slots__ = ['prec']

    def __init__(self, value=0.0, prec=12):
        self.prec = int(prec)

    def __repr__(self):
        return '%.*g' % (self.prec, self)

assert repr(precfloat(1.1)) == '1.1'
a = precfloat(12345)

assert a == 12345.0

assert float(a) == 12345.0

assert float(a).__class__ is float

assert hash(a) == hash(12345.0)

assert (+a).__class__ is float

class madcomplex(complex):

    def __repr__(self):
        return '%.17gj%+.17g' % (self.imag, self.real)
a = madcomplex(-3, 4)

assert repr(a) == '4j-3'
base = complex(-3, 4)

assert base.__class__ == complex

assert a == base

assert complex(a) == base

assert complex(a).__class__ == complex
a = madcomplex(a)

assert repr(a) == '4j-3'

assert a == base

assert complex(a) == base

assert complex(a).__class__ == complex

assert hash(a) == hash(base)

assert (+a).__class__ == complex

assert (a + 0).__class__ == complex

assert a + 0 == base

assert (a - 0).__class__ == complex

assert a - 0 == base

assert (a * 1).__class__ == complex

assert a * 1 == base

assert (a / 1).__class__ == complex

assert a / 1 == base

class madtuple(tuple):
    _rev = None

    def rev(self):
        if self._rev is not None:
            return self._rev
        L = list(self)
        L.reverse()
        self._rev = self.__class__(L)
        return self._rev
a = madtuple((1, 2, 3, 4, 5, 6, 7, 8, 9, 0))

assert a == (1, 2, 3, 4, 5, 6, 7, 8, 9, 0)

assert a.rev() == madtuple((0, 9, 8, 7, 6, 5, 4, 3, 2, 1))

assert a.rev().rev() == madtuple((1, 2, 3, 4, 5, 6, 7, 8, 9, 0))
for i in range(512):
    t = madtuple(range(i))
    u = t.rev()
    v = u.rev()

    assert v == t
a = madtuple((1, 2, 3, 4, 5))

assert tuple(a) == (1, 2, 3, 4, 5)

assert tuple(a).__class__ is tuple

assert hash(a) == hash((1, 2, 3, 4, 5))

assert a[:].__class__ is tuple

assert (a * 1).__class__ is tuple

assert (a * 0).__class__ is tuple

assert (a + ()).__class__ is tuple
a = madtuple(())

assert tuple(a) == ()

assert tuple(a).__class__ is tuple

assert (a + a).__class__ is tuple

assert (a * 0).__class__ is tuple

assert (a * 1).__class__ is tuple

assert (a * 2).__class__ is tuple

assert a[:].__class__ is tuple

class madstring(str):
    _rev = None

    def rev(self):
        if self._rev is not None:
            return self._rev
        L = list(self)
        L.reverse()
        self._rev = self.__class__(''.join(L))
        return self._rev
s = madstring('abcdefghijklmnopqrstuvwxyz')

assert s == 'abcdefghijklmnopqrstuvwxyz'

assert s.rev() == madstring('zyxwvutsrqponmlkjihgfedcba')

assert s.rev().rev() == madstring('abcdefghijklmnopqrstuvwxyz')
for i in range(256):
    s = madstring(''.join(map(chr, range(i))))
    t = s.rev()
    u = t.rev()

    assert u == s
s = madstring('12345')

assert str(s) == '12345'

assert str(s).__class__ is str
base = '\x00' * 5
s = madstring(base)

assert s == base

assert str(s) == base

assert str(s).__class__ is str

assert hash(s) == hash(base)

assert {s: 1}[base] == 1

assert {base: 1}[s] == 1

assert (s + '').__class__ is str

assert s + '' == base

assert ('' + s).__class__ is str

assert '' + s == base

assert (s * 0).__class__ is str

assert s * 0 == ''

assert (s * 1).__class__ is str

assert s * 1 == base

assert (s * 2).__class__ is str

assert s * 2 == base + base

assert s[:].__class__ is str

assert s[:] == base

assert s[0:0].__class__ is str

assert s[0:0] == ''

assert s.strip().__class__ is str

assert s.strip() == base

assert s.lstrip().__class__ is str

assert s.lstrip() == base

assert s.rstrip().__class__ is str

assert s.rstrip() == base
identitytab = {}

assert s.translate(identitytab).__class__ is str

assert s.translate(identitytab) == base

assert s.replace('x', 'x').__class__ is str

assert s.replace('x', 'x') == base

assert s.ljust(len(s)).__class__ is str

assert s.ljust(len(s)) == base

assert s.rjust(len(s)).__class__ is str

assert s.rjust(len(s)) == base

assert s.center(len(s)).__class__ is str

assert s.center(len(s)) == base

assert s.lower().__class__ is str

assert s.lower() == base

class madunicode(str):
    _rev = None

    def rev(self):
        if self._rev is not None:
            return self._rev
        L = list(self)
        L.reverse()
        self._rev = self.__class__(''.join(L))
        return self._rev
u = madunicode('ABCDEF')

assert u == 'ABCDEF'

assert u.rev() == madunicode('FEDCBA')

assert u.rev().rev() == madunicode('ABCDEF')
base = '12345'
u = madunicode(base)

assert str(u) == base

assert str(u).__class__ is str

assert hash(u) == hash(base)

assert {u: 1}[base] == 1

assert {base: 1}[u] == 1

assert u.strip().__class__ is str

assert u.strip() == base

assert u.lstrip().__class__ is str

assert u.lstrip() == base

assert u.rstrip().__class__ is str

assert u.rstrip() == base

assert u.replace('x', 'x').__class__ is str

assert u.replace('x', 'x') == base

assert u.replace('xy', 'xy').__class__ is str

assert u.replace('xy', 'xy') == base

assert u.center(len(u)).__class__ is str

assert u.center(len(u)) == base

assert u.ljust(len(u)).__class__ is str

assert u.ljust(len(u)) == base

assert u.rjust(len(u)).__class__ is str

assert u.rjust(len(u)) == base

assert u.lower().__class__ is str

assert u.lower() == base

assert u.upper().__class__ is str

assert u.upper() == base

assert u.capitalize().__class__ is str

assert u.capitalize() == base

assert u.title().__class__ is str

assert u.title() == base

assert (u + '').__class__ is str

assert u + '' == base

assert ('' + u).__class__ is str

assert '' + u == base

assert (u * 0).__class__ is str

assert u * 0 == ''

assert (u * 1).__class__ is str

assert u * 1 == base

assert (u * 2).__class__ is str

assert u * 2 == base + base

assert u[:].__class__ is str

assert u[:] == base

assert u[0:0].__class__ is str

assert u[0:0] == ''

class sublist(list):
    pass
a = sublist(range(5))

assert a == list(range(5))
a.append('hello')

assert a == list(range(5)) + ['hello']
a[5] = 5

assert a == list(range(6))
a.extend(range(6, 20))

assert a == list(range(20))
a[-5:] = []

assert a == list(range(15))
del a[10:15]

assert len(a) == 10

assert a == list(range(10))

assert list(a) == list(range(10))

assert a[0] == 0

assert a[9] == 9

assert a[-10] == 0

assert a[-1] == 9

assert a[:5] == list(range(5))
print("ClassPropertiesAndMethods::test_basic_inheritance: ok")
