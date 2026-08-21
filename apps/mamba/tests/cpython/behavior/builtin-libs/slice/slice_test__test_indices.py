# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "slice"
# dimension = "behavior"
# case = "slice_test__test_indices"
# subject = "cpython.test_slice.SliceTest.test_indices"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_slice.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_slice.py::SliceTest::test_indices
"""Auto-ported test: SliceTest::test_indices (CPython 3.12 oracle)."""


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
def check_indices(slice, length):
    try:
        actual = slice.indices(length)
    except ValueError:
        actual = 'valueerror'
    try:
        expected = slice_indices(slice, length)
    except ValueError:
        expected = 'valueerror'

    assert actual == expected
    if length >= 0 and slice.step != 0:
        actual = range(*slice.indices(length))
        expected = range(length)[slice]

        assert actual == expected

assert slice(None).indices(10) == (0, 10, 1)

assert slice(None, None, 2).indices(10) == (0, 10, 2)

assert slice(1, None, 2).indices(10) == (1, 10, 2)

assert slice(None, None, -1).indices(10) == (9, -1, -1)

assert slice(None, None, -2).indices(10) == (9, -1, -2)

assert slice(3, None, -2).indices(10) == (3, -1, -2)

assert slice(None, -9).indices(10) == (0, 1, 1)

assert slice(None, -10).indices(10) == (0, 0, 1)

assert slice(None, -11).indices(10) == (0, 0, 1)

assert slice(None, -10, -1).indices(10) == (9, 0, -1)

assert slice(None, -11, -1).indices(10) == (9, -1, -1)

assert slice(None, -12, -1).indices(10) == (9, -1, -1)

assert slice(None, 9).indices(10) == (0, 9, 1)

assert slice(None, 10).indices(10) == (0, 10, 1)

assert slice(None, 11).indices(10) == (0, 10, 1)

assert slice(None, 8, -1).indices(10) == (9, 8, -1)

assert slice(None, 9, -1).indices(10) == (9, 9, -1)

assert slice(None, 10, -1).indices(10) == (9, 9, -1)

assert slice(-100, 100).indices(10) == slice(None).indices(10)

assert slice(100, -100, -1).indices(10) == slice(None, None, -1).indices(10)

assert slice(-100, 100, 2).indices(10) == (0, 10, 2)

assert list(range(10))[::sys.maxsize - 1] == [0]
vals = [None, -2 ** 100, -2 ** 30, -53, -7, -1, 0, 1, 7, 53, 2 ** 30, 2 ** 100]
lengths = [0, 1, 7, 53, 2 ** 30, 2 ** 100]
for slice_args in itertools.product(vals, repeat=3):
    s = slice(*slice_args)
    for length in lengths:
        check_indices(s, length)
check_indices(slice(0, 10, 1), -3)
try:
    slice(None).indices(-1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    slice(0, 10, 0).indices(5)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    slice(0.0, 10, 1).indices(5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    slice(0, 10.0, 1).indices(5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    slice(0, 10, 1.0).indices(5)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    slice(0, 10, 1).indices(5.0)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert slice(0, 10, 1).indices(5) == (0, 5, 1)

assert slice(MyIndexable(0), 10, 1).indices(5) == (0, 5, 1)

assert slice(0, MyIndexable(10), 1).indices(5) == (0, 5, 1)

assert slice(0, 10, MyIndexable(1)).indices(5) == (0, 5, 1)

assert slice(0, 10, 1).indices(MyIndexable(5)) == (0, 5, 1)
print("SliceTest::test_indices: ok")
