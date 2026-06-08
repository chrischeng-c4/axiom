# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "index"
# dimension = "behavior"
# case = "list_test_case__test_setdelitem"
# subject = "cpython.test_index.ListTestCase.test_setdelitem"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_index.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_index.py::ListTestCase::test_setdelitem
"""Auto-ported test: ListTestCase::test_setdelitem (CPython 3.12 oracle)."""


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
self_o.ind = -2
self_n.ind = 2
lst = list('ab!cdefghi!j')
del lst[self_o]
del lst[self_n]
lst[self_o] = 'X'
lst[self_n] = 'Y'

assert lst == list('abYdefghXj')
lst = [5, 6, 7, 8, 9, 10, 11]
lst.__setitem__(self_n, 'here')

assert lst == [5, 6, 'here', 8, 9, 10, 11]
lst.__delitem__(self_n)

assert lst == [5, 6, 8, 9, 10, 11]
print("ListTestCase::test_setdelitem: ok")
