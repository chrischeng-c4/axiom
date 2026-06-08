# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "test_basic_ops__test_groupby"
# subject = "cpython.test_itertools.TestBasicOps.test_groupby"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_itertools.py::TestBasicOps::test_groupby
"""Auto-ported test: TestBasicOps::test_groupby (CPython 3.12 oracle)."""


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

assert [] == list(groupby([]))

assert [] == list(groupby([], key=id))

try:
    list(groupby('abc', []))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    groupby(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    groupby('abc', lambda x: x, 10)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
s = [(0, 10, 20), (0, 11, 21), (0, 12, 21), (1, 13, 21), (1, 14, 22), (2, 15, 22), (3, 16, 23), (3, 17, 23)]
dup = []
for k, g in groupby(s, lambda r: r[0]):
    for elem in g:

        assert k == elem[0]
        dup.append(elem)

assert s == dup
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    dup = []
    for k, g in pickle.loads(pickle.dumps(groupby(s, testR), proto)):
        for elem in g:

            assert k == elem[0]
            dup.append(elem)

    assert s == dup
dup = []
for k, g in groupby(s, testR):
    for ik, ig in groupby(g, testR2):
        for elem in ig:

            assert k == elem[0]

            assert ik == elem[2]
            dup.append(elem)

assert s == dup
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    dup = []
    for k, g in pickle.loads(pickle.dumps(groupby(s, testR), proto)):
        for ik, ig in pickle.loads(pickle.dumps(groupby(g, testR2), proto)):
            for elem in ig:

                assert k == elem[0]

                assert ik == elem[2]
                dup.append(elem)

    assert s == dup
keys = [k for k, g in groupby(s, testR)]
expectedkeys = set([r[0] for r in s])

assert set(keys) == expectedkeys

assert len(keys) == len(expectedkeys)
s = list(zip('AABBBAAAA', range(9)))
it = groupby(s, testR)
_, g1 = next(it)
_, g2 = next(it)
_, g3 = next(it)

assert list(g1) == []

assert list(g2) == []

assert next(g3) == ('A', 5)
list(it)

assert list(g3) == []
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    it = groupby(s, testR)
    _, g = next(it)
    next(it)
    next(it)

    assert list(pickle.loads(pickle.dumps(g, proto))) == []
s = 'abracadabra'
r = [k for k, g in groupby(sorted(s))]

assert r == ['a', 'b', 'c', 'd', 'r']
r = [k for k, g in groupby(sorted(s)) if list(islice(g, 1, 2))]

assert r == ['a', 'b', 'r']
r = [(len(list(g)), k) for k, g in groupby(sorted(s))]

assert r == [(5, 'a'), (2, 'b'), (1, 'c'), (1, 'd'), (2, 'r')]
r = sorted([(len(list(g)), k) for k, g in groupby(sorted(s))], reverse=True)[:3]

assert r == [(5, 'a'), (2, 'r'), (2, 'b')]

class ExpectedError(Exception):
    pass

def delayed_raise(n=0):
    for i in range(n):
        yield 'yo'
    raise ExpectedError

def gulp(iterable, keyp=None, func=list):
    return [func(g) for k, g in groupby(iterable, keyp)]

try:
    gulp(delayed_raise(0))
    raise AssertionError('expected ExpectedError')
except ExpectedError:
    pass

try:
    gulp(delayed_raise(1))
    raise AssertionError('expected ExpectedError')
except ExpectedError:
    pass

class DummyCmp:

    def __eq__(self, dst):
        raise ExpectedError
s = [DummyCmp(), DummyCmp(), None]

try:
    gulp(s, func=id)
    raise AssertionError('expected ExpectedError')
except ExpectedError:
    pass

try:
    gulp(s)
    raise AssertionError('expected ExpectedError')
except ExpectedError:
    pass

def keyfunc(obj):
    if keyfunc.skip > 0:
        keyfunc.skip -= 1
        return obj
    else:
        raise ExpectedError
keyfunc.skip = 0

try:
    gulp([None], keyfunc)
    raise AssertionError('expected ExpectedError')
except ExpectedError:
    pass
keyfunc.skip = 1

try:
    gulp([None, None], keyfunc)
    raise AssertionError('expected ExpectedError')
except ExpectedError:
    pass
print("TestBasicOps::test_groupby: ok")
