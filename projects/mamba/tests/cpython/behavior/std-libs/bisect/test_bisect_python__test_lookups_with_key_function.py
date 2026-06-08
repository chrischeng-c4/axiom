# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "test_bisect_python__test_lookups_with_key_function"
# subject = "cpython.test_bisect.TestBisectPython.test_lookups_with_key_function"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_bisect.py::TestBisectPython::test_lookups_with_key_function
"""Auto-ported test: TestBisectPython::test_lookups_with_key_function (CPython 3.12 oracle)."""


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
module = py_bisect
self_precomputedCases = [(module.bisect_right, [], 1, 0), (module.bisect_right, [1], 0, 0), (module.bisect_right, [1], 1, 1), (module.bisect_right, [1], 2, 1), (module.bisect_right, [1, 1], 0, 0), (module.bisect_right, [1, 1], 1, 2), (module.bisect_right, [1, 1], 2, 2), (module.bisect_right, [1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1], 1, 3), (module.bisect_right, [1, 1, 1], 2, 3), (module.bisect_right, [1, 1, 1, 1], 0, 0), (module.bisect_right, [1, 1, 1, 1], 1, 4), (module.bisect_right, [1, 1, 1, 1], 2, 4), (module.bisect_right, [1, 2], 0, 0), (module.bisect_right, [1, 2], 1, 1), (module.bisect_right, [1, 2], 1.5, 1), (module.bisect_right, [1, 2], 2, 2), (module.bisect_right, [1, 2], 3, 2), (module.bisect_right, [1, 1, 2, 2], 0, 0), (module.bisect_right, [1, 1, 2, 2], 1, 2), (module.bisect_right, [1, 1, 2, 2], 1.5, 2), (module.bisect_right, [1, 1, 2, 2], 2, 4), (module.bisect_right, [1, 1, 2, 2], 3, 4), (module.bisect_right, [1, 2, 3], 0, 0), (module.bisect_right, [1, 2, 3], 1, 1), (module.bisect_right, [1, 2, 3], 1.5, 1), (module.bisect_right, [1, 2, 3], 2, 2), (module.bisect_right, [1, 2, 3], 2.5, 2), (module.bisect_right, [1, 2, 3], 3, 3), (module.bisect_right, [1, 2, 3], 4, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 10), (module.bisect_right, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10), (module.bisect_left, [], 1, 0), (module.bisect_left, [1], 0, 0), (module.bisect_left, [1], 1, 0), (module.bisect_left, [1], 2, 1), (module.bisect_left, [1, 1], 0, 0), (module.bisect_left, [1, 1], 1, 0), (module.bisect_left, [1, 1], 2, 2), (module.bisect_left, [1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1], 2, 3), (module.bisect_left, [1, 1, 1, 1], 0, 0), (module.bisect_left, [1, 1, 1, 1], 1, 0), (module.bisect_left, [1, 1, 1, 1], 2, 4), (module.bisect_left, [1, 2], 0, 0), (module.bisect_left, [1, 2], 1, 0), (module.bisect_left, [1, 2], 1.5, 1), (module.bisect_left, [1, 2], 2, 1), (module.bisect_left, [1, 2], 3, 2), (module.bisect_left, [1, 1, 2, 2], 0, 0), (module.bisect_left, [1, 1, 2, 2], 1, 0), (module.bisect_left, [1, 1, 2, 2], 1.5, 2), (module.bisect_left, [1, 1, 2, 2], 2, 2), (module.bisect_left, [1, 1, 2, 2], 3, 4), (module.bisect_left, [1, 2, 3], 0, 0), (module.bisect_left, [1, 2, 3], 1, 0), (module.bisect_left, [1, 2, 3], 1.5, 1), (module.bisect_left, [1, 2, 3], 2, 1), (module.bisect_left, [1, 2, 3], 2.5, 2), (module.bisect_left, [1, 2, 3], 3, 2), (module.bisect_left, [1, 2, 3], 4, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 0, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1, 0), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 1.5, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2, 1), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 2.5, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3, 3), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 3.5, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 4, 6), (module.bisect_left, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4], 5, 10)]
mod = module
keyfunc = abs
arr = sorted([2, -4, 6, 8, -10], key=keyfunc)
precomputed_arr = list(map(keyfunc, arr))
for x in precomputed_arr:

    assert mod.bisect_left(arr, x, key=keyfunc) == mod.bisect_left(precomputed_arr, x)

    assert mod.bisect_right(arr, x, key=keyfunc) == mod.bisect_right(precomputed_arr, x)
keyfunc = str.casefold
arr = sorted('aBcDeEfgHhiIiij', key=keyfunc)
precomputed_arr = list(map(keyfunc, arr))
for x in precomputed_arr:

    assert mod.bisect_left(arr, x, key=keyfunc) == mod.bisect_left(precomputed_arr, x)

    assert mod.bisect_right(arr, x, key=keyfunc) == mod.bisect_right(precomputed_arr, x)
print("TestBisectPython::test_lookups_with_key_function: ok")
