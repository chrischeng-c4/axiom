# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "array_reconstructor_test__test_unicode"
# subject = "cpython.test_array.ArrayReconstructorTest.test_unicode"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_array.py::ArrayReconstructorTest::test_unicode
"""Auto-ported test: ArrayReconstructorTest::test_unicode (CPython 3.12 oracle)."""


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
teststr = 'Bonne Journée 𠌊𠍇'
testcases = ((UTF16_LE, 'UTF-16-LE'), (UTF16_BE, 'UTF-16-BE'), (UTF32_LE, 'UTF-32-LE'), (UTF32_BE, 'UTF-32-BE'))
for testcase in testcases:
    mformat_code, encoding = testcase
    a = array.array('u', teststr)
    b = array_reconstructor(array.array, 'u', mformat_code, teststr.encode(encoding))

    assert a == b
print("ArrayReconstructorTest::test_unicode: ok")
