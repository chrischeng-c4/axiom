# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "unsigned_long_test__test_count"
# subject = "cpython.test_array.UnsignedLongTest.test_count"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_array.py::UnsignedLongTest::test_count
"""Auto-ported test: UnsignedLongTest::test_count (CPython 3.12 oracle)."""


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
example = [0, 1, 17, 23, 42, 255]
smallerexample = [0, 1, 17, 23, 42, 254]
biggerexample = [0, 1, 17, 23, 43, 255]
outside = 170
typecode = 'L'
minitemsize = 4

def assertEntryEqual(entry1, entry2):

    assert entry1 == entry2

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
    a.count()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
for x in example:

    assert a.count(x) == example.count(x)

assert a.count(outside) == 0

assert a.count(None) == 0
print("UnsignedLongTest::test_count: ok")
