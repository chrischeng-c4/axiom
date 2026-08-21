# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "index"
# dimension = "behavior"
# case = "tuple_test_case__test_slice_bug7532"
# subject = "cpython.test_index.TupleTestCase.test_slice_bug7532"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_index.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_index.py::TupleTestCase::test_slice_bug7532
"""Auto-ported test: TupleTestCase::test_slice_bug7532 (CPython 3.12 oracle)."""


import unittest
from test import support
import operator


maxsize = support.MAX_Py_ssize_t

class newstyle:

    def __index__(self):
        return self.ind

class TrapInt(int):

    def __index__(self):
        return int(self)

class NewSeq:

    def __init__(self, iterable):
        self._list = list(iterable)

    def __repr__(self):
        return repr(self._list)

    def __eq__(self, other):
        return self._list == other

    def __len__(self):
        return len(self._list)

    def __mul__(self, n):
        return self.__class__(self._list * n)
    __rmul__ = __mul__

    def __getitem__(self, index):
        return self._list[index]


# --- test body ---
seq = (0, 10, 20, 30, 40, 50)
self_o = newstyle()
self_n = newstyle()
self_o2 = newstyle()
self_n2 = newstyle()
seqlen = len(seq)
self_o.ind = int(seqlen * 1.5)
self_n.ind = seqlen + 2

assert seq[self_o:] == seq[0:0]

assert seq[:self_o] == seq

assert seq[self_n:] == seq[0:0]

assert seq[:self_n] == seq
self_o2.ind = -seqlen - 2
self_n2.ind = -int(seqlen * 1.5)

assert seq[self_o2:] == seq

assert seq[:self_o2] == seq[0:0]

assert seq[self_n2:] == seq

assert seq[:self_n2] == seq[0:0]
print("TupleTestCase::test_slice_bug7532: ok")
