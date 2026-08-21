# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "test_basic_ops__test_compress"
# subject = "cpython.test_itertools.TestBasicOps.test_compress"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_itertools.py::TestBasicOps::test_compress
"""Auto-ported test: TestBasicOps::test_compress (CPython 3.12 oracle)."""


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

assert list(compress(data='ABCDEF', selectors=[1, 0, 1, 0, 1, 1])) == list('ACEF')

assert list(compress('ABCDEF', [1, 0, 1, 0, 1, 1])) == list('ACEF')

assert list(compress('ABCDEF', [0, 0, 0, 0, 0, 0])) == list('')

assert list(compress('ABCDEF', [1, 1, 1, 1, 1, 1])) == list('ABCDEF')

assert list(compress('ABCDEF', [1, 0, 1])) == list('AC')

assert list(compress('ABC', [0, 1, 1, 1, 1, 1])) == list('BC')
n = 10000
data = chain.from_iterable(repeat(range(6), n))
selectors = chain.from_iterable(repeat((0, 1)))

assert list(compress(data, selectors)) == [1, 3, 5] * n

try:
    compress(None, range(6))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    compress(range(6), None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    compress(range(6))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    compress(range(6), None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
for op in [lambda a: copy.copy(a), lambda a: copy.deepcopy(a)] + picklecopiers:
    for data, selectors, result1, result2 in [('ABCDEF', [1, 0, 1, 0, 1, 1], 'ACEF', 'CEF'), ('ABCDEF', [0, 0, 0, 0, 0, 0], '', ''), ('ABCDEF', [1, 1, 1, 1, 1, 1], 'ABCDEF', 'BCDEF'), ('ABCDEF', [1, 0, 1], 'AC', 'C'), ('ABC', [0, 1, 1, 1, 1, 1], 'BC', 'C')]:

        assert list(op(compress(data=data, selectors=selectors))) == list(result1)

        assert list(op(compress(data, selectors))) == list(result1)
        testIntermediate = compress(data, selectors)
        if result1:
            next(testIntermediate)

            assert list(op(testIntermediate)) == list(result2)
print("TestBasicOps::test_compress: ok")
