# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "test_pure_python_rough_equivalents__test_islice_recipe"
# subject = "cpython.test_itertools.TestPurePythonRoughEquivalents.test_islice_recipe"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_itertools.py::TestPurePythonRoughEquivalents::test_islice_recipe
"""Auto-ported test: TestPurePythonRoughEquivalents::test_islice_recipe (CPython 3.12 oracle)."""


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
def islice(iterable, *args):
    s = slice(*args)
    start, stop, step = (s.start or 0, s.stop or sys.maxsize, s.step or 1)
    it = iter(range(start, stop, step))
    try:
        nexti = next(it)
    except StopIteration:
        for i, element in zip(range(start), iterable):
            pass
        return
    try:
        for i, element in enumerate(iterable):
            if i == nexti:
                yield element
                nexti = next(it)
    except StopIteration:
        for i, element in zip(range(i + 1, stop), iterable):
            pass

assert list(islice('ABCDEFG', 2)) == list('AB')

assert list(islice('ABCDEFG', 2, 4)) == list('CD')

assert list(islice('ABCDEFG', 2, None)) == list('CDEFG')

assert list(islice('ABCDEFG', 0, None, 2)) == list('ACEG')
it = iter(range(10))

assert list(islice(it, 3)) == list(range(3))

assert list(it) == list(range(3, 10))
it = iter(range(10))

assert list(islice(it, 3, 3)) == []

assert list(it) == list(range(3, 10))
c = count()

assert list(islice(c, 1, 3, 50)) == [1]

assert next(c) == 3
print("TestPurePythonRoughEquivalents::test_islice_recipe: ok")
