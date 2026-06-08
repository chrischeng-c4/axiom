# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "index"
# dimension = "behavior"
# case = "list_test_case__test_inplace_repeat"
# subject = "cpython.test_index.ListTestCase.test_inplace_repeat"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_index.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_index.py::ListTestCase::test_inplace_repeat
"""Auto-ported test: ListTestCase::test_inplace_repeat (CPython 3.12 oracle)."""


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
seq = [0, 10, 20, 30, 40, 50]
self_o = newstyle()
self_n = newstyle()
self_o2 = newstyle()
self_n2 = newstyle()
self_o.ind = 2
self_n.ind = 3
lst = [6, 4]
lst *= self_o

assert lst == [6, 4, 6, 4]
lst *= self_n

assert lst == [6, 4, 6, 4] * 3
lst = [5, 6, 7, 8, 9, 11]
l2 = lst.__imul__(self_n)

assert l2 is lst

assert lst == [5, 6, 7, 8, 9, 11] * 3
print("ListTestCase::test_inplace_repeat: ok")
