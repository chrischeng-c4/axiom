# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "index"
# dimension = "behavior"
# case = "tuple_test_case__test_repeat"
# subject = "cpython.test_index.TupleTestCase.test_repeat"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_index.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_index.py::TupleTestCase::test_repeat
"""Auto-ported test: TupleTestCase::test_repeat (CPython 3.12 oracle)."""


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
self_o.ind = 3
self_n.ind = 2

assert seq * self_o == seq * 3

assert seq * self_n == seq * 2

assert self_o * seq == seq * 3

assert self_n * seq == seq * 2
print("TupleTestCase::test_repeat: ok")
