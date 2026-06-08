# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "double_test__test_index"
# subject = "cpython.test_array.DoubleTest.test_index"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_array.py::DoubleTest::test_index
"""Auto-ported test: DoubleTest::test_index (CPython 3.12 oracle)."""


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
example = [-42.0, 0, 42, 100000.0, -10000000000.0]
smallerexample = [-42.0, 0, 42, 100000.0, -20000000000.0]
biggerexample = [-42.0, 0, 42, 100000.0, 10000000000.0]
outside = 23
typecode = 'd'
minitemsize = 8

def assertEntryEqual(entry1, entry2):

    assert abs(entry1 - entry2) < 1e-07

def badtypecode():
    return typecodes[(typecodes.index(typecode) + 1) % len(typecodes)]

def check_overflow(lower, upper):
    a = array.array(typecode, [lower])
    a[0] = lower

    try:
        array.array(typecode, [lower - 1])
        raise AssertionError('expected OverflowError')
    except OverflowError:
        pass

    try:
        a.__setitem__(0, lower - 1)
        raise AssertionError('expected OverflowError')
    except OverflowError:
        pass
    a = array.array(typecode, [upper])
    a[0] = upper

    try:
        array.array(typecode, [upper + 1])
        raise AssertionError('expected OverflowError')
    except OverflowError:
        pass

    try:
        a.__setitem__(0, upper + 1)
        raise AssertionError('expected OverflowError')
    except OverflowError:
        pass
example = 2 * example
a = array.array(typecode, example)

try:
    a.index()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
for x in example:

    assert a.index(x) == example.index(x)

try:
    a.index(None)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    a.index(outside)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
a = array.array('i', [-2, -1, 0, 0, 1, 2])

assert a.index(0) == 2

assert a.index(0, 2) == 2

assert a.index(0, -4) == 2

assert a.index(-2, -10) == 0

assert a.index(0, 3) == 3

assert a.index(0, -3) == 3

assert a.index(0, 3, 4) == 3

assert a.index(0, -3, -2) == 3

try:
    a.index(2, 0, -10)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("DoubleTest::test_index: ok")
