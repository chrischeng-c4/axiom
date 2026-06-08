# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "index"
# dimension = "behavior"
# case = "new_seq_test_case__test_slice"
# subject = "cpython.test_index.NewSeqTestCase.test_slice"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_index.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_index.py::NewSeqTestCase::test_slice
"""Auto-ported test: NewSeqTestCase::test_slice (CPython 3.12 oracle)."""


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
seq = NewSeq((0, 10, 20, 30, 40, 50))
self_o = newstyle()
self_n = newstyle()
self_o2 = newstyle()
self_n2 = newstyle()
self_o.ind = 1
self_o2.ind = 3
self_n.ind = 2
self_n2.ind = 4

assert seq[self_o:self_o2] == seq[1:3]

assert seq[self_n:self_n2] == seq[2:4]
print("NewSeqTestCase::test_slice: ok")
