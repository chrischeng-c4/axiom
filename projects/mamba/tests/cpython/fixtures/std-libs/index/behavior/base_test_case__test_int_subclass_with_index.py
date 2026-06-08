# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "index"
# dimension = "behavior"
# case = "base_test_case__test_int_subclass_with_index"
# subject = "cpython.test_index.BaseTestCase.test_int_subclass_with_index"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_index.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_index.py::BaseTestCase::test_int_subclass_with_index
"""Auto-ported test: BaseTestCase::test_int_subclass_with_index (CPython 3.12 oracle)."""


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

class MyInt(int):

    def __index__(self):
        return int(str(self)) + 1
my_int = MyInt(7)
direct_index = my_int.__index__()
operator_index = operator.index(my_int)

assert direct_index == 8

assert operator_index == 7

assert type(direct_index) is int
print("BaseTestCase::test_int_subclass_with_index: ok")
