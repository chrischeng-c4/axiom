# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "index"
# dimension = "behavior"
# case = "base_test_case__test_slice"
# subject = "cpython.test_index.BaseTestCase.test_slice"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_index.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_index.py::BaseTestCase::test_slice
"""Auto-ported test: BaseTestCase::test_slice (CPython 3.12 oracle)."""


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
self_o = newstyle()
self_n = newstyle()
self_o.ind = 1
self_n.ind = 2
slc = slice(self_o, self_o, self_o)
check_slc = slice(1, 1, 1)

assert slc.indices(self_o) == check_slc.indices(1)
slc = slice(self_n, self_n, self_n)
check_slc = slice(2, 2, 2)

assert slc.indices(self_n) == check_slc.indices(2)
print("BaseTestCase::test_slice: ok")
