# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "iter"
# dimension = "behavior"
# case = "test_case__test_count_of"
# subject = "cpython.test_iter.TestCase.test_countOf"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_iter.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_iter.py::TestCase::test_countOf
"""Auto-ported test: TestCase::test_countOf (CPython 3.12 oracle)."""


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
from operator import countOf

assert countOf([1, 2, 2, 3, 2, 5], 2) == 3

assert countOf((1, 2, 2, 3, 2, 5), 2) == 3

assert countOf('122325', '2') == 3

assert countOf('122325', '6') == 0

try:
    countOf(42, 1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    countOf(countOf, countOf)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
d = {'one': 3, 'two': 3, 'three': 3, 1j: 2j}
for k in d:

    assert countOf(d, k) == 1

assert countOf(d.values(), 3) == 3

assert countOf(d.values(), 2j) == 1

assert countOf(d.values(), 1j) == 0
f = open(TESTFN, 'w', encoding='utf-8')
try:
    f.write('a\nb\nc\nb\n')
finally:
    f.close()
f = open(TESTFN, 'r', encoding='utf-8')
try:
    for letter, count in (('a', 1), ('b', 2), ('c', 1), ('d', 0)):
        f.seek(0, 0)

        assert countOf(f, letter + '\n') == count
finally:
    f.close()
    try:
        unlink(TESTFN)
    except OSError:
        pass
print("TestCase::test_countOf: ok")
