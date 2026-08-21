# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "array_reconstructor_test__test_numbers"
# subject = "cpython.test_array.ArrayReconstructorTest.test_numbers"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_array.py::ArrayReconstructorTest::test_numbers
"""Auto-ported test: ArrayReconstructorTest::test_numbers (CPython 3.12 oracle)."""


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
testcases = ((['B', 'H', 'I', 'L'], UNSIGNED_INT8, '=BBBB', [128, 127, 0, 255]), (['b', 'h', 'i', 'l'], SIGNED_INT8, '=bbb', [-128, 127, 0]), (['H', 'I', 'L'], UNSIGNED_INT16_LE, '<HHHH', [32768, 32767, 0, 65535]), (['H', 'I', 'L'], UNSIGNED_INT16_BE, '>HHHH', [32768, 32767, 0, 65535]), (['h', 'i', 'l'], SIGNED_INT16_LE, '<hhh', [-32768, 32767, 0]), (['h', 'i', 'l'], SIGNED_INT16_BE, '>hhh', [-32768, 32767, 0]), (['I', 'L'], UNSIGNED_INT32_LE, '<IIII', [1 << 31, (1 << 31) - 1, 0, (1 << 32) - 1]), (['I', 'L'], UNSIGNED_INT32_BE, '>IIII', [1 << 31, (1 << 31) - 1, 0, (1 << 32) - 1]), (['i', 'l'], SIGNED_INT32_LE, '<iii', [-1 << 31, (1 << 31) - 1, 0]), (['i', 'l'], SIGNED_INT32_BE, '>iii', [-1 << 31, (1 << 31) - 1, 0]), (['L'], UNSIGNED_INT64_LE, '<QQQQ', [1 << 31, (1 << 31) - 1, 0, (1 << 32) - 1]), (['L'], UNSIGNED_INT64_BE, '>QQQQ', [1 << 31, (1 << 31) - 1, 0, (1 << 32) - 1]), (['l'], SIGNED_INT64_LE, '<qqq', [-1 << 31, (1 << 31) - 1, 0]), (['l'], SIGNED_INT64_BE, '>qqq', [-1 << 31, (1 << 31) - 1, 0]), (['L'], UNSIGNED_INT64_LE, '<QQQQ', [1 << 63, (1 << 63) - 1, 0, (1 << 64) - 1]), (['L'], UNSIGNED_INT64_BE, '>QQQQ', [1 << 63, (1 << 63) - 1, 0, (1 << 64) - 1]), (['l'], SIGNED_INT64_LE, '<qqq', [-1 << 63, (1 << 63) - 1, 0]), (['l'], SIGNED_INT64_BE, '>qqq', [-1 << 63, (1 << 63) - 1, 0]), (['f'], IEEE_754_FLOAT_LE, '<ffff', [16711938.0, float('inf'), float('-inf'), -0.0]), (['f'], IEEE_754_FLOAT_BE, '>ffff', [16711938.0, float('inf'), float('-inf'), -0.0]), (['d'], IEEE_754_DOUBLE_LE, '<dddd', [9006104071832581.0, float('inf'), float('-inf'), -0.0]), (['d'], IEEE_754_DOUBLE_BE, '>dddd', [9006104071832581.0, float('inf'), float('-inf'), -0.0]))
for testcase in testcases:
    valid_typecodes, mformat_code, struct_fmt, values = testcase
    arraystr = struct.pack(struct_fmt, *values)
    for typecode in valid_typecodes:
        try:
            a = array.array(typecode, values)
        except OverflowError:
            continue
        b = array_reconstructor(array.array, typecode, mformat_code, arraystr)

        assert a == b
print("ArrayReconstructorTest::test_numbers: ok")
