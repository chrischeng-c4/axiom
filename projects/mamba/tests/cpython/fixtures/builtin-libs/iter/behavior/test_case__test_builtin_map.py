# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "iter"
# dimension = "behavior"
# case = "test_case__test_builtin_map"
# subject = "cpython.test_iter.TestCase.test_builtin_map"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_iter.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_iter.py::TestCase::test_builtin_map
"""Auto-ported test: TestCase::test_builtin_map (CPython 3.12 oracle)."""


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

assert list(map(lambda x: x + 1, SequenceClass(5))) == list(range(1, 6))
d = {'one': 1, 'two': 2, 'three': 3}

assert list(map(lambda k, d=d: (k, d[k]), d)) == list(d.items())
dkeys = list(d.keys())
expected = [(i < len(d) and dkeys[i] or None, i, i < len(d) and dkeys[i] or None) for i in range(3)]
f = open(TESTFN, 'w', encoding='utf-8')
try:
    for i in range(10):
        f.write('xy' * i + '\n')
finally:
    f.close()
f = open(TESTFN, 'r', encoding='utf-8')
try:

    assert list(map(len, f)) == list(range(1, 21, 2))
finally:
    f.close()
    try:
        unlink(TESTFN)
    except OSError:
        pass
print("TestCase::test_builtin_map: ok")
