# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "fraction_test__test_int_subclass"
# subject = "cpython.test_fractions.FractionTest.test_int_subclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_fractions.py::FractionTest::test_int_subclass
"""Auto-ported test: FractionTest::test_int_subclass (CPython 3.12 oracle)."""


import cmath
from decimal import Decimal
from test.support import requires_IEEE_754
import math
import numbers
import operator
import fractions
import functools
import os
import sys
import typing
import unittest
from copy import copy, deepcopy
import pickle
from pickle import dumps, loads


'Tests for Lib/fractions.py.'

F = fractions.Fraction

test_dir = os.path.dirname(__file__) or os.curdir

format_testfile = os.path.join(test_dir, 'formatfloat_testcases.txt')

class DummyFloat(object):
    """Dummy float class for testing comparisons with Fractions"""

    def __init__(self, value):
        if not isinstance(value, float):
            raise TypeError('DummyFloat can only be initialized from float')
        self.value = value

    def _richcmp(self, other, op):
        if isinstance(other, numbers.Rational):
            return op(F.from_float(self.value), other)
        elif isinstance(other, DummyFloat):
            return op(self.value, other.value)
        else:
            return NotImplemented

    def __eq__(self, other):
        return self._richcmp(other, operator.eq)

    def __le__(self, other):
        return self._richcmp(other, operator.le)

    def __lt__(self, other):
        return self._richcmp(other, operator.lt)

    def __ge__(self, other):
        return self._richcmp(other, operator.ge)

    def __gt__(self, other):
        return self._richcmp(other, operator.gt)

    def __float__(self):
        assert False, '__float__ should not be invoked for comparisons'

    def __sub__(self, other):
        assert False, '__sub__ should not be invoked for comparisons'
    __rsub__ = __sub__

class DummyRational(object):
    """Test comparison of Fraction with a naive rational implementation."""

    def __init__(self, num, den):
        g = math.gcd(num, den)
        self.num = num // g
        self.den = den // g

    def __eq__(self, other):
        if isinstance(other, fractions.Fraction):
            return self.num == other._numerator and self.den == other._denominator
        else:
            return NotImplemented

    def __lt__(self, other):
        return self.num * other._denominator < self.den * other._numerator

    def __gt__(self, other):
        return self.num * other._denominator > self.den * other._numerator

    def __le__(self, other):
        return self.num * other._denominator <= self.den * other._numerator

    def __ge__(self, other):
        return self.num * other._denominator >= self.den * other._numerator

    def __float__(self):
        assert False, '__float__ should not be invoked'

class DummyFraction(fractions.Fraction):
    """Dummy Fraction subclass for copy and deepcopy testing."""

def _components(r):
    return (r.numerator, r.denominator)

def typed_approx_eq(a, b):
    return type(a) == type(b) and (a == b or math.isclose(a, b))

class Symbolic:
    """Simple non-numeric class for testing mixed arithmetic.
    It is not Integral, Rational, Real or Complex, and cannot be converted
    to int, float or complex. but it supports some arithmetic operations.
    """

    def __init__(self, value):
        self.value = value

    def __mul__(self, other):
        if isinstance(other, F):
            return NotImplemented
        return self.__class__(f'{self} * {other}')

    def __rmul__(self, other):
        return self.__class__(f'{other} * {self}')

    def __truediv__(self, other):
        if isinstance(other, F):
            return NotImplemented
        return self.__class__(f'{self} / {other}')

    def __rtruediv__(self, other):
        return self.__class__(f'{other} / {self}')

    def __mod__(self, other):
        if isinstance(other, F):
            return NotImplemented
        return self.__class__(f'{self} % {other}')

    def __rmod__(self, other):
        return self.__class__(f'{other} % {self}')

    def __pow__(self, other):
        if isinstance(other, F):
            return NotImplemented
        return self.__class__(f'{self} ** {other}')

    def __rpow__(self, other):
        return self.__class__(f'{other} ** {self}')

    def __eq__(self, other):
        if other.__class__ != self.__class__:
            return NotImplemented
        return self.value == other.value

    def __str__(self):
        return f'{self.value}'

    def __repr__(self):
        return f'{self.__class__.__name__}({self.value!r})'

class SymbolicReal(Symbolic):
    pass

numbers.Real.register(SymbolicReal)

class SymbolicComplex(Symbolic):
    pass

numbers.Complex.register(SymbolicComplex)

class Rat:
    """Simple Rational class for testing mixed arithmetic."""

    def __init__(self, n, d):
        self.numerator = n
        self.denominator = d

    def __mul__(self, other):
        if isinstance(other, F):
            return NotImplemented
        return self.__class__(self.numerator * other.numerator, self.denominator * other.denominator)

    def __rmul__(self, other):
        return self.__class__(other.numerator * self.numerator, other.denominator * self.denominator)

    def __truediv__(self, other):
        if isinstance(other, F):
            return NotImplemented
        return self.__class__(self.numerator * other.denominator, self.denominator * other.numerator)

    def __rtruediv__(self, other):
        return self.__class__(other.numerator * self.denominator, other.denominator * self.numerator)

    def __mod__(self, other):
        if isinstance(other, F):
            return NotImplemented
        d = self.denominator * other.numerator
        return self.__class__(self.numerator * other.denominator % d, d)

    def __rmod__(self, other):
        d = other.denominator * self.numerator
        return self.__class__(other.numerator * self.denominator % d, d)
        return self.__class__(other.numerator / self.numerator, other.denominator / self.denominator)

    def __pow__(self, other):
        if isinstance(other, F):
            return NotImplemented
        return self.__class__(self.numerator ** other, self.denominator ** other)

    def __float__(self):
        return self.numerator / self.denominator

    def __eq__(self, other):
        if self.__class__ != other.__class__:
            return NotImplemented
        return typed_approx_eq(self.numerator, other.numerator) and typed_approx_eq(self.denominator, other.denominator)

    def __repr__(self):
        return f'{self.__class__.__name__}({self.numerator!r}, {self.denominator!r})'

numbers.Rational.register(Rat)

class Root:
    """Simple Real class for testing mixed arithmetic."""

    def __init__(self, v, n=F(2)):
        self.base = v
        self.degree = n

    def __mul__(self, other):
        if isinstance(other, F):
            return NotImplemented
        return self.__class__(self.base * other ** self.degree, self.degree)

    def __rmul__(self, other):
        return self.__class__(other ** self.degree * self.base, self.degree)

    def __truediv__(self, other):
        if isinstance(other, F):
            return NotImplemented
        return self.__class__(self.base / other ** self.degree, self.degree)

    def __rtruediv__(self, other):
        return self.__class__(other ** self.degree / self.base, self.degree)

    def __pow__(self, other):
        if isinstance(other, F):
            return NotImplemented
        return self.__class__(self.base, self.degree / other)

    def __float__(self):
        return float(self.base) ** (1 / float(self.degree))

    def __eq__(self, other):
        if self.__class__ != other.__class__:
            return NotImplemented
        return typed_approx_eq(self.base, other.base) and typed_approx_eq(self.degree, other.degree)

    def __repr__(self):
        return f'{self.__class__.__name__}({self.base!r}, {self.degree!r})'

numbers.Real.register(Root)

class Polar:
    """Simple Complex class for testing mixed arithmetic."""

    def __init__(self, r, phi):
        self.r = r
        self.phi = phi

    def __mul__(self, other):
        if isinstance(other, F):
            return NotImplemented
        return self.__class__(self.r * other, self.phi)

    def __rmul__(self, other):
        return self.__class__(other * self.r, self.phi)

    def __truediv__(self, other):
        if isinstance(other, F):
            return NotImplemented
        return self.__class__(self.r / other, self.phi)

    def __rtruediv__(self, other):
        return self.__class__(other / self.r, -self.phi)

    def __pow__(self, other):
        if isinstance(other, F):
            return NotImplemented
        return self.__class__(self.r ** other, self.phi * other)

    def __eq__(self, other):
        if self.__class__ != other.__class__:
            return NotImplemented
        return typed_approx_eq(self.r, other.r) and typed_approx_eq(self.phi, other.phi)

    def __repr__(self):
        return f'{self.__class__.__name__}({self.r!r}, {self.phi!r})'

numbers.Complex.register(Polar)

class Rect:
    """Other simple Complex class for testing mixed arithmetic."""

    def __init__(self, x, y):
        self.x = x
        self.y = y

    def __mul__(self, other):
        if isinstance(other, F):
            return NotImplemented
        return self.__class__(self.x * other, self.y * other)

    def __rmul__(self, other):
        return self.__class__(other * self.x, other * self.y)

    def __truediv__(self, other):
        if isinstance(other, F):
            return NotImplemented
        return self.__class__(self.x / other, self.y / other)

    def __rtruediv__(self, other):
        r = self.x * self.x + self.y * self.y
        return self.__class__(other * (self.x / r), other * (self.y / r))

    def __rpow__(self, other):
        return Polar(other ** self.x, math.log(other) * self.y)

    def __complex__(self):
        return complex(self.x, self.y)

    def __eq__(self, other):
        if self.__class__ != other.__class__:
            return NotImplemented
        return typed_approx_eq(self.x, other.x) and typed_approx_eq(self.y, other.y)

    def __repr__(self):
        return f'{self.__class__.__name__}({self.x!r}, {self.y!r})'

numbers.Complex.register(Rect)

class RectComplex(Rect, complex):
    pass


# --- test body ---
class myint(int):

    def __mul__(self, other):
        return type(self)(int(self) * int(other))

    def __floordiv__(self, other):
        return type(self)(int(self) // int(other))

    def __mod__(self, other):
        x = type(self)(int(self) % int(other))
        return x

    @property
    def numerator(self):
        return type(self)(int(self))

    @property
    def denominator(self):
        return type(self)(1)
f = fractions.Fraction(myint(1 * 3), myint(2 * 3))

assert f.numerator == 1

assert f.denominator == 2

assert type(f.numerator) == myint

assert type(f.denominator) == myint
print("FractionTest::test_int_subclass: ok")
