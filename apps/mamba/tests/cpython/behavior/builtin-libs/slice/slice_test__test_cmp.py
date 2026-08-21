# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "slice"
# dimension = "behavior"
# case = "slice_test__test_cmp"
# subject = "cpython.test_slice.SliceTest.test_cmp"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_slice.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_slice.py::SliceTest::test_cmp
"""Auto-ported test: SliceTest::test_cmp (CPython 3.12 oracle)."""


import itertools
import operator
import sys
import unittest
import weakref
import copy
from pickle import loads, dumps
from test import support


def evaluate_slice_index(arg):
    """
    Helper function to convert a slice argument to an integer, and raise
    TypeError with a suitable message on failure.

    """
    if hasattr(arg, '__index__'):
        return operator.index(arg)
    else:
        raise TypeError('slice indices must be integers or None or have an __index__ method')

def slice_indices(slice, length):
    """
    Reference implementation for the slice.indices method.

    """
    length = operator.index(length)
    step = 1 if slice.step is None else evaluate_slice_index(slice.step)
    if length < 0:
        raise ValueError('length should not be negative')
    if step == 0:
        raise ValueError('slice step cannot be zero')
    lower = -1 if step < 0 else 0
    upper = length - 1 if step < 0 else length
    if slice.start is None:
        start = upper if step < 0 else lower
    else:
        start = evaluate_slice_index(slice.start)
        start = max(start + length, lower) if start < 0 else min(start, upper)
    if slice.stop is None:
        stop = lower if step < 0 else upper
    else:
        stop = evaluate_slice_index(slice.stop)
        stop = max(stop + length, lower) if stop < 0 else min(stop, upper)
    return (start, stop, step)

class MyIndexable(object):

    def __init__(self, value):
        self.value = value

    def __index__(self):
        return self.value


# --- test body ---
s1 = slice(1, 2, 3)
s2 = slice(1, 2, 3)
s3 = slice(1, 2, 4)

assert s1 == s2

assert s1 != s3

assert s1 != None

assert s1 != (1, 2, 3)

assert s1 != ''

class Exc(Exception):
    pass

class BadCmp(object):

    def __eq__(self, other):
        raise Exc
s1 = slice(BadCmp())
s2 = slice(BadCmp())

assert s1 == s1

try:
    (lambda: s1 == s2)()
    raise AssertionError('expected Exc')
except Exc:
    pass
s1 = slice(1, BadCmp())
s2 = slice(1, BadCmp())

assert s1 == s1

try:
    (lambda: s1 == s2)()
    raise AssertionError('expected Exc')
except Exc:
    pass
s1 = slice(1, 2, BadCmp())
s2 = slice(1, 2, BadCmp())

assert s1 == s1

try:
    (lambda: s1 == s2)()
    raise AssertionError('expected Exc')
except Exc:
    pass
print("SliceTest::test_cmp: ok")
