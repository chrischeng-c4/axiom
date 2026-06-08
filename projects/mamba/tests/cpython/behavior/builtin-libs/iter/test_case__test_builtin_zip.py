# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "iter"
# dimension = "behavior"
# case = "test_case__test_builtin_zip"
# subject = "cpython.test_iter.TestCase.test_builtin_zip"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_iter.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_iter.py::TestCase::test_builtin_zip
"""Auto-ported test: TestCase::test_builtin_zip (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import cpython_only
from test.support.os_helper import TESTFN, unlink
from test.support import check_free_after_iterating, ALWAYS_EQ, NEVER_EQ
from test.support import BrokenIter
import pickle
import collections.abc
import functools
import contextlib
import builtins
import traceback


TRIPLETS = [(0, 0, 0), (0, 0, 1), (0, 0, 2), (0, 1, 0), (0, 1, 1), (0, 1, 2), (0, 2, 0), (0, 2, 1), (0, 2, 2), (1, 0, 0), (1, 0, 1), (1, 0, 2), (1, 1, 0), (1, 1, 1), (1, 1, 2), (1, 2, 0), (1, 2, 1), (1, 2, 2), (2, 0, 0), (2, 0, 1), (2, 0, 2), (2, 1, 0), (2, 1, 1), (2, 1, 2), (2, 2, 0), (2, 2, 1), (2, 2, 2)]

class BasicIterClass:

    def __init__(self, n):
        self.n = n
        self.i = 0

    def __next__(self):
        res = self.i
        if res >= self.n:
            raise StopIteration
        self.i = res + 1
        return res

    def __iter__(self):
        return self

class IteratingSequenceClass:

    def __init__(self, n):
        self.n = n

    def __iter__(self):
        return BasicIterClass(self.n)

class IteratorProxyClass:

    def __init__(self, i):
        self.i = i

    def __next__(self):
        return next(self.i)

    def __iter__(self):
        return self

class SequenceClass:

    def __init__(self, n):
        self.n = n

    def __getitem__(self, i):
        if 0 <= i < self.n:
            return i
        else:
            raise IndexError

class SequenceProxyClass:

    def __init__(self, s):
        self.s = s

    def __getitem__(self, i):
        return self.s[i]

class UnlimitedSequenceClass:

    def __getitem__(self, i):
        return i

class DefaultIterClass:
    pass

class NoIterClass:

    def __getitem__(self, i):
        return i
    __iter__ = None

class BadIterableClass:

    def __iter__(self):
        raise ZeroDivisionError

class CallableIterClass:

    def __init__(self):
        self.i = 0

    def __call__(self):
        i = self.i
        self.i = i + 1
        if i > 100:
            raise IndexError
        return i

class EmptyIterClass:

    def __len__(self):
        return 0

    def __getitem__(self, i):
        raise StopIteration


# --- test body ---

assert list(zip()) == []

assert list(zip(*[])) == []

assert list(zip(*[(1, 2), 'ab'])) == [(1, 'a'), (2, 'b')]

try:
    zip(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    zip(range(10), 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    zip(range(10), zip)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert list(zip(IteratingSequenceClass(3))) == [(0,), (1,), (2,)]

assert list(zip(SequenceClass(3))) == [(0,), (1,), (2,)]
d = {'one': 1, 'two': 2, 'three': 3}

assert list(d.items()) == list(zip(d, d.values()))

class IntsFrom:

    def __init__(self, start):
        self.i = start

    def __iter__(self):
        return self

    def __next__(self):
        i = self.i
        self.i = i + 1
        return i
f = open(TESTFN, 'w', encoding='utf-8')
try:
    f.write('a\nbbb\ncc\n')
finally:
    f.close()
f = open(TESTFN, 'r', encoding='utf-8')
try:

    assert list(zip(IntsFrom(0), f, IntsFrom(-100))) == [(0, 'a\n', -100), (1, 'bbb\n', -99), (2, 'cc\n', -98)]
finally:
    f.close()
    try:
        unlink(TESTFN)
    except OSError:
        pass

assert list(zip(range(5))) == [(i,) for i in range(5)]

class NoGuessLen5:

    def __getitem__(self, i):
        if i >= 5:
            raise IndexError
        return i

class Guess3Len5(NoGuessLen5):

    def __len__(self):
        return 3

class Guess30Len5(NoGuessLen5):

    def __len__(self):
        return 30

def lzip(*args):
    return list(zip(*args))

assert len(Guess3Len5()) == 3

assert len(Guess30Len5()) == 30

assert lzip(NoGuessLen5()) == lzip(range(5))

assert lzip(Guess3Len5()) == lzip(range(5))

assert lzip(Guess30Len5()) == lzip(range(5))
expected = [(i, i) for i in range(5)]
for x in (NoGuessLen5(), Guess3Len5(), Guess30Len5()):
    for y in (NoGuessLen5(), Guess3Len5(), Guess30Len5()):

        assert lzip(x, y) == expected
print("TestCase::test_builtin_zip: ok")
