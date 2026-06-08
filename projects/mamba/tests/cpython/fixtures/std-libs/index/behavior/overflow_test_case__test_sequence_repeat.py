# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "index"
# dimension = "behavior"
# case = "overflow_test_case__test_sequence_repeat"
# subject = "cpython.test_index.OverflowTestCase.test_sequence_repeat"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_index.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_index.py::OverflowTestCase::test_sequence_repeat
"""Auto-ported test: OverflowTestCase::test_sequence_repeat (CPython 3.12 oracle)."""


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
self_pos = 2 ** 100
self_neg = -self_pos

try:
    (lambda: 'a' * self_pos)()
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

try:
    (lambda: 'a' * self_neg)()
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass
print("OverflowTestCase::test_sequence_repeat: ok")
