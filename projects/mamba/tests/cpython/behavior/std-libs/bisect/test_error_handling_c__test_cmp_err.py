# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_error_handling_c__test_cmp_err"
# subject = "cpython.test_bisect.TestErrorHandlingC.test_cmp_err"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bisect.py::TestErrorHandlingC::test_cmp_err
"""Auto-ported test: TestErrorHandlingC::test_cmp_err (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper
from collections import UserList


py_bisect = import_helper.import_fresh_module('bisect', blocked=['_bisect'])

c_bisect = import_helper.import_fresh_module('bisect', fresh=['_bisect'])

class Range(object):
    """A trivial range()-like object that has an insert() method."""

    def __init__(self, start, stop):
        self.start = start
        self.stop = stop
        self.last_insert = None

    def __len__(self):
        return self.stop - self.start

    def __getitem__(self, idx):
        n = self.stop - self.start
        if idx < 0:
            idx += n
        if idx >= n:
            raise IndexError(idx)
        return self.start + idx

    def insert(self, idx, item):
        self.last_insert = (idx, item)

class LenOnly:
    """Dummy sequence class defining __len__ but not __getitem__."""

    def __len__(self):
        return 10

class GetOnly:
    """Dummy sequence class defining __getitem__ but not __len__."""

    def __getitem__(self, ndx):
        return 10

class CmpErr:
    """Dummy element that always raises an error during comparison"""

    def __lt__(self, other):
        raise ZeroDivisionError
    __gt__ = __lt__
    __le__ = __lt__
    __ge__ = __lt__
    __eq__ = __lt__
    __ne__ = __lt__


# --- test body ---
module = c_bisect
seq = [CmpErr(), CmpErr(), CmpErr()]
for f in (module.bisect_left, module.bisect_right, module.insort_left, module.insort_right):

    try:
        f(seq, 10)
        raise AssertionError('expected ZeroDivisionError')
    except ZeroDivisionError:
        pass
print("TestErrorHandlingC::test_cmp_err: ok")
