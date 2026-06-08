# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "iter"
# dimension = "behavior"
# case = "test_case__test_unpack_iter"
# subject = "cpython.test_iter.TestCase.test_unpack_iter"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_iter.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_iter.py::TestCase::test_unpack_iter
"""Auto-ported test: TestCase::test_unpack_iter (CPython 3.12 oracle)."""


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
a, b = (1, 2)

assert (a, b) == (1, 2)
a, b, c = IteratingSequenceClass(3)

assert (a, b, c) == (0, 1, 2)
try:
    a, b = IteratingSequenceClass(3)
except ValueError:
    pass
else:

    raise AssertionError('should have raised ValueError')
try:
    a, b, c = IteratingSequenceClass(2)
except ValueError:
    pass
else:

    raise AssertionError('should have raised ValueError')
try:
    a, b, c = len
except TypeError:
    pass
else:

    raise AssertionError('should have raised TypeError')
a, b, c = {1: 42, 2: 42, 3: 42}.values()

assert (a, b, c) == (42, 42, 42)
f = open(TESTFN, 'w', encoding='utf-8')
lines = ('a\n', 'bb\n', 'ccc\n')
try:
    for line in lines:
        f.write(line)
finally:
    f.close()
f = open(TESTFN, 'r', encoding='utf-8')
try:
    a, b, c = f

    assert (a, b, c) == lines
finally:
    f.close()
    try:
        unlink(TESTFN)
    except OSError:
        pass
(a, b), (c,) = (IteratingSequenceClass(2), {42: 24})

assert (a, b, c) == (0, 1, 42)
print("TestCase::test_unpack_iter: ok")
