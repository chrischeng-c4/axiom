# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "long_test__test_reverse_iterator_picking"
# subject = "cpython.test_array.LongTest.test_reverse_iterator_picking"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_array.py::LongTest::test_reverse_iterator_picking
"""Auto-ported test: LongTest::test_reverse_iterator_picking (CPython 3.12 oracle)."""


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
typecode = 'l'
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
orig = array.array(typecode, example)
data = list(orig)
data2 = [outside] + data
rev_data = data[len(data) - 2::-1] + [outside]
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    itorig = reversed(orig)
    d = pickle.dumps((itorig, orig), proto)
    it, a = pickle.loads(d)
    a.insert(0, outside)

    assert type(it) == type(itorig)

    assert list(it) == rev_data

    assert list(a) == data2
    next(itorig)
    d = pickle.dumps((itorig, orig), proto)
    it, a = pickle.loads(d)
    a.insert(0, outside)

    assert type(it) == type(itorig)

    assert list(it) == rev_data[1:]

    assert list(a) == data2
    for i in range(1, len(data)):
        next(itorig)
    d = pickle.dumps((itorig, orig), proto)
    it, a = pickle.loads(d)
    a.insert(0, outside)

    assert type(it) == type(itorig)

    assert list(it) == []

    assert list(a) == data2

    try:
        next(itorig)
        raise AssertionError('expected StopIteration')
    except StopIteration:
        pass
    d = pickle.dumps((itorig, orig), proto)
    it, a = pickle.loads(d)
    a.insert(0, outside)

    assert list(it) == []

    assert list(a) == data2
print("LongTest::test_reverse_iterator_picking: ok")
