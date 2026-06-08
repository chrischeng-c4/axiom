# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "index"
# dimension = "behavior"
# case = "base_test_case__test_error"
# subject = "cpython.test_index.BaseTestCase.test_error"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_index.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_index.py::BaseTestCase::test_error
"""Auto-ported test: BaseTestCase::test_error (CPython 3.12 oracle)."""


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
self_o.ind = 'dumb'
self_n.ind = 'bad'

try:
    operator.index(self_o)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    operator.index(self_n)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    slice(self_o).indices(0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    slice(self_n).indices(0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("BaseTestCase::test_error: ok")
