# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "iter"
# dimension = "behavior"
# case = "test_case__test_builtin_filter"
# subject = "cpython.test_iter.TestCase.test_builtin_filter"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_iter.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_iter.py::TestCase::test_builtin_filter
"""Auto-ported test: TestCase::test_builtin_filter (CPython 3.12 oracle)."""


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

assert list(filter(None, SequenceClass(5))) == list(range(1, 5))

assert list(filter(None, SequenceClass(0))) == []

assert list(filter(None, ())) == []

assert list(filter(None, 'abc')) == ['a', 'b', 'c']
d = {'one': 1, 'two': 2, 'three': 3}

assert list(filter(None, d)) == list(d.keys())

try:
    filter(None, list)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    filter(None, 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class Boolean:

    def __init__(self, truth):
        self.truth = truth

    def __bool__(self):
        return self.truth
bTrue = Boolean(True)
bFalse = Boolean(False)

class Seq:

    def __init__(self, *args):
        self.vals = args

    def __iter__(self):

        class SeqIter:

            def __init__(self, vals):
                self.vals = vals
                self.i = 0

            def __iter__(self):
                return self

            def __next__(self):
                i = self.i
                self.i = i + 1
                if i < len(self.vals):
                    return self.vals[i]
                else:
                    raise StopIteration
        return SeqIter(self.vals)
seq = Seq(*[bTrue, bFalse] * 25)

assert list(filter(lambda x: not x, seq)) == [bFalse] * 25

assert list(filter(lambda x: not x, iter(seq))) == [bFalse] * 25
print("TestCase::test_builtin_filter: ok")
