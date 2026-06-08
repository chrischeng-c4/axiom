# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "iter"
# dimension = "behavior"
# case = "test_case__test_in_and_not_in"
# subject = "cpython.test_iter.TestCase.test_in_and_not_in"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_iter.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_iter.py::TestCase::test_in_and_not_in
"""Auto-ported test: TestCase::test_in_and_not_in (CPython 3.12 oracle)."""


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
for sc5 in (IteratingSequenceClass(5), SequenceClass(5)):
    for i in range(5):

        assert i in sc5
    for i in ('abc', -1, 5, 42.42, (3, 4), [], {1: 1}, 3 - 12j, sc5):

        assert i not in sc5

assert ALWAYS_EQ in IteratorProxyClass(iter([1]))

assert ALWAYS_EQ in SequenceProxyClass([1])

assert ALWAYS_EQ not in IteratorProxyClass(iter([NEVER_EQ]))

assert ALWAYS_EQ not in SequenceProxyClass([NEVER_EQ])

assert NEVER_EQ in IteratorProxyClass(iter([ALWAYS_EQ]))

assert NEVER_EQ in SequenceProxyClass([ALWAYS_EQ])

try:
    (lambda: 3 in 12)()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    (lambda: 3 not in map)()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    (lambda: 3 in BadIterableClass())()
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass
d = {'one': 1, 'two': 2, 'three': 3, 1j: 2j}
for k in d:

    assert k in d

    assert k not in d.values()
for v in d.values():

    assert v in d.values()

    assert v not in d
for k, v in d.items():

    assert (k, v) in d.items()

    assert (v, k) not in d.items()
f = open(TESTFN, 'w', encoding='utf-8')
try:
    f.write('a\nb\nc\n')
finally:
    f.close()
f = open(TESTFN, 'r', encoding='utf-8')
try:
    for chunk in 'abc':
        f.seek(0, 0)

        assert chunk not in f
        f.seek(0, 0)

        assert chunk + '\n' in f
finally:
    f.close()
    try:
        unlink(TESTFN)
    except OSError:
        pass
print("TestCase::test_in_and_not_in: ok")
