# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "byte_test__test_buffer_info"
# subject = "cpython.test_array.ByteTest.test_buffer_info"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_array.py::ByteTest::test_buffer_info
"""Auto-ported test: ByteTest::test_buffer_info (CPython 3.12 oracle)."""


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
example = [-1, 0, 1, 42, 127]
smallerexample = [-1, 0, 1, 42, 126]
biggerexample = [-1, 0, 1, 43, 127]
outside = 23
typecode = 'b'
minitemsize = 1

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
a = array.array(typecode, example)

try:
    a.buffer_info(42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
bi = a.buffer_info()

assert isinstance(bi, tuple)

assert len(bi) == 2

assert isinstance(bi[0], int)

assert isinstance(bi[1], int)

assert bi[1] == len(a)
print("ByteTest::test_buffer_info: ok")
