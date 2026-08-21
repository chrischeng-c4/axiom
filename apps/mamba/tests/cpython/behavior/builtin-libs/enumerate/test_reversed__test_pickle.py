# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "enumerate"
# dimension = "behavior"
# case = "test_reversed__test_pickle"
# subject = "cpython.test.test_enumerate.TestReversed.test_pickle"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_enumerate.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_enumerate.py::TestReversed::test_pickle
"""Auto-ported test: TestReversed::test_pickle (CPython 3.12 oracle)."""


import unittest
import operator
import sys
import pickle
import gc
from test import support


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

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class PickleTest:

    def check_pickle(self, itorg, seq):
        for proto in range(pickle.HIGHEST_PROTOCOL + 1):
            d = pickle.dumps(itorg, proto)
            it = pickle.loads(d)
            self.assertEqual(type(itorg), type(it))
            self.assertEqual(list(it), seq)
            it = pickle.loads(d)
            try:
                next(it)
            except StopIteration:
                self.assertFalse(seq[1:])
                continue
            d = pickle.dumps(it, proto)
            it = pickle.loads(d)
            self.assertEqual(list(it), seq[1:])

class MyEnum(enumerate):
    pass


# --- test body ---
def check_pickle(itorg, seq):
    for proto in range(pickle.HIGHEST_PROTOCOL + 1):
        d = pickle.dumps(itorg, proto)
        it = pickle.loads(d)

        assert type(itorg) == type(it)

        assert list(it) == seq
        it = pickle.loads(d)
        try:
            next(it)
        except StopIteration:

            assert not seq[1:]
            continue
        d = pickle.dumps(it, proto)
        it = pickle.loads(d)

        assert list(it) == seq[1:]
for data in ('abc', range(5), tuple(enumerate('abc')), range(1, 17, 5)):
    check_pickle(reversed(data), list(data)[::-1])
print("TestReversed::test_pickle: ok")
