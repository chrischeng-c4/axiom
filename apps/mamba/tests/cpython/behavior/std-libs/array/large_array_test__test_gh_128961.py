# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "large_array_test__test_gh_128961"
# subject = "cpython.test_array.LargeArrayTest.test_gh_128961"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_array.py::LargeArrayTest::test_gh_128961
"""Auto-ported test: LargeArrayTest::test_gh_128961 (CPython 3.12 oracle)."""


import collections.abc
import unittest
from test import support
from test.support import import_helper
from test.support import os_helper
from test.support import _2G
import weakref
import pickle
import operator
import struct
import sys
import array
from array import _array_reconstructor as array_reconstructor


'Test the arraymodule.\n   Roger E. Masse\n'

sizeof_wchar = array.array('u').itemsize

class ArraySubclass(array.array):
    pass

class ArraySubclassWithKwargs(array.array):

    def __init__(self, typecode, newarg=None):
        array.array.__init__(self)

typecodes = 'ubBhHiIlLfdqQ'

UNKNOWN_FORMAT = -1

UNSIGNED_INT8 = 0

SIGNED_INT8 = 1

UNSIGNED_INT16_LE = 2

UNSIGNED_INT16_BE = 3

SIGNED_INT16_LE = 4

SIGNED_INT16_BE = 5

UNSIGNED_INT32_LE = 6

UNSIGNED_INT32_BE = 7

SIGNED_INT32_LE = 8

SIGNED_INT32_BE = 9

UNSIGNED_INT64_LE = 10

UNSIGNED_INT64_BE = 11

SIGNED_INT64_LE = 12

SIGNED_INT64_BE = 13

IEEE_754_FLOAT_LE = 14

IEEE_754_FLOAT_BE = 15

IEEE_754_DOUBLE_LE = 16

IEEE_754_DOUBLE_BE = 17

UTF16_LE = 18

UTF16_BE = 19

UTF32_LE = 20

UTF32_BE = 21

class Intable:

    def __init__(self, num):
        self._num = num

    def __index__(self):
        return self._num

    def __int__(self):
        return self._num

    def __sub__(self, other):
        return Intable(int(self) - int(other))

    def __add__(self, other):
        return Intable(int(self) + int(other))


# --- test body ---
typecode = 'b'

def example(size):
    base = array.array(typecode, [0, 1, 2, 3, 4, 5, 6, 7]) * (size // 8)
    base += array.array(typecode, [99] * (size % 8) + [8, 9, 10, 11])
    return base
a = array.array('i')
it = iter(a)
list(it)
it.__setstate__(0)

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass
print("LargeArrayTest::test_gh_128961: ok")
