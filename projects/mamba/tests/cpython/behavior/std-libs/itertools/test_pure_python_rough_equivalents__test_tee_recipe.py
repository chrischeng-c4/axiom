# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "test_pure_python_rough_equivalents__test_tee_recipe"
# subject = "cpython.test_itertools.TestPurePythonRoughEquivalents.test_tee_recipe"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_itertools.py::TestPurePythonRoughEquivalents::test_tee_recipe
"""Auto-ported test: TestPurePythonRoughEquivalents::test_tee_recipe (CPython 3.12 oracle)."""


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
def tee(iterable, n=2):
    if n < 0:
        raise ValueError
    if n == 0:
        return ()
    iterator = _tee(iterable)
    result = [iterator]
    for _ in range(n - 1):
        result.append(_tee(iterator))
    return tuple(result)

class _tee:

    def __init__(self, iterable):
        it = iter(iterable)
        if isinstance(it, _tee):
            self.iterator = it.iterator
            self.link = it.link
        else:
            self.iterator = it
            self.link = [None, None]

    def __iter__(self):
        return self

    def __next__(self):
        link = self.link
        if link[1] is None:
            link[0] = next(self.iterator)
            link[1] = [None, None]
        value, self.link = link
        return value
n = 200
a, b = tee([])

assert list(a) == []

assert list(b) == []
a, b = tee(irange(n))

assert lzip(a, b) == lzip(range(n), range(n))
a, b = tee(irange(n))

assert list(a) == list(range(n))

assert list(b) == list(range(n))
a, b = tee(irange(n))
for i in range(100):

    assert next(a) == i
del a

assert list(b) == list(range(n))
a, b = tee(irange(n))
for i in range(100):

    assert next(a) == i
del b

assert list(a) == list(range(100, n))
for j in range(5):
    order = [0] * n + [1] * n
    random.shuffle(order)
    lists = ([], [])
    its = tee(irange(n))
    for i in order:
        value = next(its[i])
        lists[i].append(value)

    assert lists[0] == list(range(n))

    assert lists[1] == list(range(n))

try:
    tee()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    tee(3)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    tee([1, 2], 'x')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    tee([1, 2], 3, 'x')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
a, b = tee('abc')
c = type(a)('def')

assert list(c) == list('def')
a, b, c = tee(range(2000), 3)
for i in range(100):

    assert next(a) == i

assert list(b) == list(range(2000))

assert [next(c), next(c)] == list(range(2))

assert list(a) == list(range(100, 2000))

assert list(c) == list(range(2, 2000))

try:
    tee('abc', 'invalid')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    tee([], -1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
for n in range(5):
    result = tee('abc', n)

    assert type(result) == tuple

    assert len(result) == n

    assert [list(x) for x in result] == [list('abc')] * n
a, b = tee('abc')
c, d = tee(a)
e, f = tee(c)

assert len({a, b, c, d, e, f}) == 6
t1, t2 = tee('abc')
tnew = type(t1)

try:
    tnew()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    tnew(10)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
t3 = tnew(t1)

assert list(t1) == list(t2) == list(t3) == list('abc')
a, b = tee(range(10))
p = weakref.proxy(a)

assert getattr(p, '__class__') == type(b)
del a
gc.collect()

try:
    getattr(p, '__class__')
    raise AssertionError('expected ReferenceError')
except ReferenceError:
    pass
ans = list('abc')
long_ans = list(range(10000))
if False:
    a, b = tee('abc')

    assert list(copy.copy(a)) == ans

    assert list(copy.copy(b)) == ans
    a, b = tee(list(range(10000)))

    assert list(copy.copy(a)) == long_ans

    assert list(copy.copy(b)) == long_ans
    a, b = tee('abc')
    take(2, a)
    take(1, b)

    assert list(copy.copy(a)) == ans[2:]

    assert list(copy.copy(b)) == ans[1:]

    assert list(a) == ans[2:]

    assert list(b) == ans[1:]
    a, b = tee(range(10000))
    take(100, a)
    take(60, b)

    assert list(copy.copy(a)) == long_ans[100:]

    assert list(copy.copy(b)) == long_ans[60:]

    assert list(a) == long_ans[100:]

    assert list(b) == long_ans[60:]
forward, backward = tee(repeat(None, 2000))
try:
    any(forward)
    del backward
except:
    del forward, backward
    raise
print("TestPurePythonRoughEquivalents::test_tee_recipe: ok")
