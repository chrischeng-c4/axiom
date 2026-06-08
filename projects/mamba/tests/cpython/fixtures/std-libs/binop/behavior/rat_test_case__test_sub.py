# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binop"
# dimension = "behavior"
# case = "rat_test_case__test_sub"
# subject = "cpython.test_binop.RatTestCase.test_sub"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_binop.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_binop.py::RatTestCase::test_sub
"""Auto-ported test: RatTestCase::test_sub (CPython 3.12 oracle)."""


import unittest
from operator import eq, le, ne
from abc import ABCMeta


'Tests for binary operators on subtypes of built-in types.'

def gcd(a, b):
    """Greatest common divisor using Euclid's algorithm."""
    while a:
        a, b = (b % a, a)
    return b

def isint(x):
    """Test whether an object is an instance of int."""
    return isinstance(x, int)

def isnum(x):
    """Test whether an object is an instance of a built-in numeric type."""
    for T in (int, float, complex):
        if isinstance(x, T):
            return 1
    return 0

def isRat(x):
    """Test whether an object is an instance of the Rat class."""
    return isinstance(x, Rat)

class Rat(object):
    """Rational number implemented as a normalized pair of ints."""
    __slots__ = ['_Rat__num', '_Rat__den']

    def __init__(self, num=0, den=1):
        """Constructor: Rat([num[, den]]).

        The arguments must be ints, and default to (0, 1)."""
        if not isint(num):
            raise TypeError('Rat numerator must be int (%r)' % num)
        if not isint(den):
            raise TypeError('Rat denominator must be int (%r)' % den)
        if den == 0:
            raise ZeroDivisionError('zero denominator')
        g = gcd(den, num)
        self.__num = int(num // g)
        self.__den = int(den // g)

    def _get_num(self):
        """Accessor function for read-only 'num' attribute of Rat."""
        return self.__num
    num = property(_get_num, None)

    def _get_den(self):
        """Accessor function for read-only 'den' attribute of Rat."""
        return self.__den
    den = property(_get_den, None)

    def __repr__(self):
        """Convert a Rat to a string resembling a Rat constructor call."""
        return 'Rat(%d, %d)' % (self.__num, self.__den)

    def __str__(self):
        """Convert a Rat to a string resembling a decimal numeric value."""
        return str(float(self))

    def __float__(self):
        """Convert a Rat to a float."""
        return self.__num * 1.0 / self.__den

    def __int__(self):
        """Convert a Rat to an int; self.den must be 1."""
        if self.__den == 1:
            try:
                return int(self.__num)
            except OverflowError:
                raise OverflowError('%s too large to convert to int' % repr(self))
        raise ValueError("can't convert %s to int" % repr(self))

    def __add__(self, other):
        """Add two Rats, or a Rat and a number."""
        if isint(other):
            other = Rat(other)
        if isRat(other):
            return Rat(self.__num * other.__den + other.__num * self.__den, self.__den * other.__den)
        if isnum(other):
            return float(self) + other
        return NotImplemented
    __radd__ = __add__

    def __sub__(self, other):
        """Subtract two Rats, or a Rat and a number."""
        if isint(other):
            other = Rat(other)
        if isRat(other):
            return Rat(self.__num * other.__den - other.__num * self.__den, self.__den * other.__den)
        if isnum(other):
            return float(self) - other
        return NotImplemented

    def __rsub__(self, other):
        """Subtract two Rats, or a Rat and a number (reversed args)."""
        if isint(other):
            other = Rat(other)
        if isRat(other):
            return Rat(other.__num * self.__den - self.__num * other.__den, self.__den * other.__den)
        if isnum(other):
            return other - float(self)
        return NotImplemented

    def __mul__(self, other):
        """Multiply two Rats, or a Rat and a number."""
        if isRat(other):
            return Rat(self.__num * other.__num, self.__den * other.__den)
        if isint(other):
            return Rat(self.__num * other, self.__den)
        if isnum(other):
            return float(self) * other
        return NotImplemented
    __rmul__ = __mul__

    def __truediv__(self, other):
        """Divide two Rats, or a Rat and a number."""
        if isRat(other):
            return Rat(self.__num * other.__den, self.__den * other.__num)
        if isint(other):
            return Rat(self.__num, self.__den * other)
        if isnum(other):
            return float(self) / other
        return NotImplemented

    def __rtruediv__(self, other):
        """Divide two Rats, or a Rat and a number (reversed args)."""
        if isRat(other):
            return Rat(other.__num * self.__den, other.__den * self.__num)
        if isint(other):
            return Rat(other * self.__den, self.__num)
        if isnum(other):
            return other / float(self)
        return NotImplemented

    def __floordiv__(self, other):
        """Divide two Rats, returning the floored result."""
        if isint(other):
            other = Rat(other)
        elif not isRat(other):
            return NotImplemented
        x = self / other
        return x.__num // x.__den

    def __rfloordiv__(self, other):
        """Divide two Rats, returning the floored result (reversed args)."""
        x = other / self
        return x.__num // x.__den

    def __divmod__(self, other):
        """Divide two Rats, returning quotient and remainder."""
        if isint(other):
            other = Rat(other)
        elif not isRat(other):
            return NotImplemented
        x = self // other
        return (x, self - other * x)

    def __rdivmod__(self, other):
        """Divide two Rats, returning quotient and remainder (reversed args)."""
        if isint(other):
            other = Rat(other)
        elif not isRat(other):
            return NotImplemented
        return divmod(other, self)

    def __mod__(self, other):
        """Take one Rat modulo another."""
        return divmod(self, other)[1]

    def __rmod__(self, other):
        """Take one Rat modulo another (reversed args)."""
        return divmod(other, self)[1]

    def __eq__(self, other):
        """Compare two Rats for equality."""
        if isint(other):
            return self.__den == 1 and self.__num == other
        if isRat(other):
            return self.__num == other.__num and self.__den == other.__den
        if isnum(other):
            return float(self) == other
        return NotImplemented

class OperationLogger:
    """Base class for classes with operation logging."""

    def __init__(self, logger):
        self.logger = logger

    def log_operation(self, *args):
        self.logger(*args)

def op_sequence(op, *classes):
    """Return the sequence of operations that results from applying
    the operation `op` to instances of the given classes."""
    log = []
    instances = []
    for c in classes:
        instances.append(c(log.append))
    try:
        op(*instances)
    except TypeError:
        pass
    return log

class A(OperationLogger):

    def __eq__(self, other):
        self.log_operation('A.__eq__')
        return NotImplemented

    def __le__(self, other):
        self.log_operation('A.__le__')
        return NotImplemented

    def __ge__(self, other):
        self.log_operation('A.__ge__')
        return NotImplemented

class B(OperationLogger, metaclass=ABCMeta):

    def __eq__(self, other):
        self.log_operation('B.__eq__')
        return NotImplemented

    def __le__(self, other):
        self.log_operation('B.__le__')
        return NotImplemented

    def __ge__(self, other):
        self.log_operation('B.__ge__')
        return NotImplemented

class C(B):

    def __eq__(self, other):
        self.log_operation('C.__eq__')
        return NotImplemented

    def __le__(self, other):
        self.log_operation('C.__le__')
        return NotImplemented

    def __ge__(self, other):
        self.log_operation('C.__ge__')
        return NotImplemented

class V(OperationLogger):
    """Virtual subclass of B"""

    def __eq__(self, other):
        self.log_operation('V.__eq__')
        return NotImplemented

    def __le__(self, other):
        self.log_operation('V.__le__')
        return NotImplemented

    def __ge__(self, other):
        self.log_operation('V.__ge__')
        return NotImplemented

B.register(V)

class SupEq(object):
    """Class that can test equality"""

    def __eq__(self, other):
        return True

class S(SupEq):
    """Subclass of SupEq that should fail"""
    __eq__ = None

class F(object):
    """Independent class that should fall back"""

class X(object):
    """Independent class that should fail"""
    __eq__ = None

class SN(SupEq):
    """Subclass of SupEq that can test equality, but not non-equality"""
    __ne__ = None

class XN:
    """Independent class that can test equality, but not non-equality"""

    def __eq__(self, other):
        return True
    __ne__ = None


# --- test body ---

assert Rat(7, 2) - Rat(7, 5) == Rat(21, 10)

assert Rat(7, 5) - 1 == Rat(2, 5)

assert 1 - Rat(3, 5) == Rat(2, 5)

assert Rat(3, 2) - 1.0 == 0.5

assert 1.0 - Rat(1, 2) == 0.5
print("RatTestCase::test_sub: ok")
