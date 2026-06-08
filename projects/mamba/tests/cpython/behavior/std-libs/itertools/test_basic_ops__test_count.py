# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "test_basic_ops__test_count"
# subject = "cpython.test_itertools.TestBasicOps.test_count"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_itertools.py::TestBasicOps::test_count
"""Auto-ported test: TestBasicOps::test_count (CPython 3.12 oracle)."""


import doctest
import unittest
import itertools
from test import support
from test.support import threading_helper, script_helper
from itertools import *
import weakref
from decimal import Decimal
from fractions import Fraction
import operator
import random
import copy
import pickle
from functools import reduce
import sys
import struct
import threading
import gc
import warnings


def pickle_deprecated(testfunc):
    """ Run the test three times.
    First, verify that a Deprecation Warning is raised.
    Second, run normally but with DeprecationWarnings temporarily disabled.
    Third, run with warnings promoted to errors.
    """

    def inner(self):
        with self.assertWarns(DeprecationWarning):
            testfunc(self)
        with warnings.catch_warnings():
            warnings.simplefilter('ignore', category=DeprecationWarning)
            testfunc(self)
        with warnings.catch_warnings():
            warnings.simplefilter('error', category=DeprecationWarning)
            with self.assertRaises((DeprecationWarning, AssertionError, SystemError)):
                testfunc(self)
    return inner

maxsize = support.MAX_Py_ssize_t

minsize = -maxsize - 1

def lzip(*args):
    return list(zip(*args))

def onearg(x):
    """Test function of one argument"""
    return 2 * x

def errfunc(*args):
    """Test function that raises an error"""
    raise ValueError

def gen3():
    """Non-restartable source sequence"""
    for i in (0, 1, 2):
        yield i

def isEven(x):
    """Test predicate"""
    return x % 2 == 0

def isOdd(x):
    """Test predicate"""
    return x % 2 == 1

def tupleize(*args):
    return args

def irange(n):
    for i in range(n):
        yield i

class StopNow:
    """Class emulating an empty iterable."""

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def take(n, seq):
    """Convenience function for partially consuming a long of infinite iterable"""
    return list(islice(seq, n))

def prod(iterable):
    return reduce(operator.mul, iterable, 1)

def fact(n):
    """Factorial"""
    return prod(range(1, n + 1))

def testR(r):
    return r[0]

def testR2(r):
    return r[2]

def underten(x):
    return x < 10

picklecopiers = [lambda s, proto=proto: pickle.loads(pickle.dumps(s, proto)) for proto in range(pickle.HIGHEST_PROTOCOL + 1)]

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class E2:
    """Test propagation of exceptions after two iterations"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i == 2:
            raise ZeroDivisionError
        v = self.seqn[self.i]
        self.i += 1
        return v

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(itertools))
    return tests


# --- test body ---
def pickletest(protocol, it, stop=4, take=1, compare=None):
    """Test that an iterator is the same after pickling, also when part-consumed"""

    def expand(it, i=0):
        if i > 10:
            raise RuntimeError('infinite recursion encountered')
        if isinstance(it, str):
            return it
        try:
            l = list(islice(it, stop))
        except TypeError:
            return it
        return [expand(e, i + 1) for e in l]
    dump = pickle.dumps(it, protocol)
    i2 = pickle.loads(dump)

    assert type(it) == type(i2)
    a, b = (expand(it), expand(i2))

    assert a == b
    if compare:
        c = expand(compare)

        assert a == c
    i3 = pickle.loads(dump)
    took = 0
    try:
        for i in range(take):
            next(i3)
            took += 1
    except StopIteration:
        pass
    dump = pickle.dumps(i3, protocol)
    i4 = pickle.loads(dump)
    a, b = (expand(i3), expand(i4))

    assert a == b
    if compare:
        c = expand(compare[took:])

        assert a == c

assert lzip('abc', count()) == [('a', 0), ('b', 1), ('c', 2)]

assert lzip('abc', count(3)) == [('a', 3), ('b', 4), ('c', 5)]

assert take(2, lzip('abc', count(3))) == [('a', 3), ('b', 4)]

assert take(2, zip('abc', count(-1))) == [('a', -1), ('b', 0)]

assert take(2, zip('abc', count(-3))) == [('a', -3), ('b', -2)]

try:
    count(2, 3, 4)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    count('a')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert take(3, count(maxsize)) == [maxsize, maxsize + 1, maxsize + 2]

assert take(10, count(maxsize - 5)) == list(range(maxsize - 5, maxsize + 5))

assert take(10, count(-maxsize - 5)) == list(range(-maxsize - 5, -maxsize + 5))

assert take(3, count(3.25)) == [3.25, 4.25, 5.25]

assert take(3, count(3.25 - 4j)) == [3.25 - 4j, 4.25 - 4j, 5.25 - 4j]

assert take(3, count(Decimal('1.1'))) == [Decimal('1.1'), Decimal('2.1'), Decimal('3.1')]

assert take(3, count(Fraction(2, 3))) == [Fraction(2, 3), Fraction(5, 3), Fraction(8, 3)]
BIGINT = 1 << 1000

assert take(3, count(BIGINT)) == [BIGINT, BIGINT + 1, BIGINT + 2]
c = count(3)

assert repr(c) == 'count(3)'
next(c)

assert repr(c) == 'count(4)'
c = count(-9)

assert repr(c) == 'count(-9)'
next(c)

assert next(c) == -8

assert repr(count(10.25)) == 'count(10.25)'

assert repr(count(10.0)) == 'count(10.0)'

assert repr(count(maxsize)) == f'count({maxsize})'
c = count(maxsize - 1)

assert repr(c) == f'count({maxsize - 1})'
next(c)

assert repr(c) == f'count({maxsize})'
next(c)

assert repr(c) == f'count({maxsize + 1})'

assert type(next(count(10.0))) == float
for i in (-sys.maxsize - 5, -sys.maxsize + 5, -10, -1, 0, 10, sys.maxsize - 5, sys.maxsize + 5):
    r1 = repr(count(i))
    r2 = 'count(%r)'.__mod__(i)

    assert r1 == r2
for value in (-3, 3, maxsize - 5, maxsize + 5):
    c = count(value)

    assert next(copy.copy(c)) == value

    assert next(copy.deepcopy(c)) == value
    for proto in range(pickle.HIGHEST_PROTOCOL + 1):
        pickletest(proto, count(value))
count(1, maxsize + 5)
sys.exc_info()
print("TestBasicOps::test_count: ok")
