# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "iter"
# dimension = "behavior"
# case = "test_case__test_reduce_mutating_builtins_iter"
# subject = "cpython.test_iter.TestCase.test_reduce_mutating_builtins_iter"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_iter.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_iter.py::TestCase::test_reduce_mutating_builtins_iter
"""Auto-ported test: TestCase::test_reduce_mutating_builtins_iter (CPython 3.12 oracle)."""


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
builtins_dict = builtins.__dict__
orig = {'iter': iter, 'reversed': reversed}

def run(builtin_name, item, sentinel=None):
    it = iter(item) if sentinel is None else iter(item, sentinel)

    class CustomStr:

        def __init__(self, name, iterator):
            self.name = name
            self.iterator = iterator

        def __hash__(self):
            return hash(self.name)

        def __eq__(self, other):
            list(self.iterator)
            return other == self.name
    del builtins_dict[builtin_name]
    builtins_dict[CustomStr(builtin_name, it)] = orig[builtin_name]
    return it.__reduce__()
types = [(EmptyIterClass(),), (bytes(8),), (bytearray(8),), ((1, 2, 3),), (lambda: 0, 0), (tuple[int],)]
try:
    run_iter = functools.partial(run, 'iter')

    assert run_iter('xyz') == (orig['iter'], ('',))

    assert run_iter([1, 2, 3]) == (orig['iter'], ([],))

    assert run('reversed', orig['reversed'](list(range(8)))) == (reversed, ([],))
    for case in types:

        assert run_iter(*case) == (orig['iter'], ((),))
finally:
    for key, func in orig.items():
        with contextlib.suppress(KeyError):
            del builtins_dict[key]
        builtins_dict[key] = func
print("TestCase::test_reduce_mutating_builtins_iter: ok")
