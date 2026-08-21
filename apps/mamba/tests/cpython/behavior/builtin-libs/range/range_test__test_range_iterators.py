# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_range_iterators"
# subject = "cpython.test_range.RangeTest.test_range_iterators"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_range_iterators
"""Auto-ported test: RangeTest::test_range_iterators (CPython 3.12 oracle)."""


import unittest
import sys
import pickle
import itertools
from test.support import ALWAYS_EQ


def pyrange(start, stop, step):
    if (start - stop) // step < 0:
        stop += (start - stop) % step
        while start != stop:
            yield start
            start += step

def pyrange_reversed(start, stop, step):
    stop += (start - stop) % step
    return pyrange(stop - step, start - step, -step)


# --- test body ---
def assert_attrs(rangeobj, start, stop, step):

    assert rangeobj.start == start

    assert rangeobj.stop == stop

    assert rangeobj.step == step

    assert type(rangeobj.start) is int

    assert type(rangeobj.stop) is int

    assert type(rangeobj.step) is int
    try:
        rangeobj.start = 0
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass
    try:
        rangeobj.stop = 10
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass
    try:
        rangeobj.step = 1
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass
    try:
        del rangeobj.start
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass
    try:
        del rangeobj.stop
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass
    try:
        del rangeobj.step
        raise AssertionError('expected AttributeError')
    except AttributeError:
        pass

def assert_iterators_equal(xs, ys, test_id, limit=None):
    if limit is not None:
        xs = itertools.islice(xs, limit)
        ys = itertools.islice(ys, limit)
    sentinel = object()
    pairs = itertools.zip_longest(xs, ys, fillvalue=sentinel)
    for i, (x, y) in enumerate(pairs):
        if x == y:
            continue
        elif x == sentinel:

            raise AssertionError('{}: iterator ended unexpectedly at position {}; expected {}'.format(test_id, i, y))
        elif y == sentinel:

            raise AssertionError('{}: unexpected excess element {} at position {}'.format(test_id, x, i))
        else:

            raise AssertionError('{}: wrong element at position {}; expected {}, got {}'.format(test_id, i, y, x))
limits = [base + jiggle for M in (2 ** 32, 2 ** 64) for base in (-M, -M // 2, 0, M // 2, M) for jiggle in (-2, -1, 0, 1, 2)]
test_ranges = [(start, end, step) for start in limits for end in limits for step in (-2 ** 63, -2 ** 31, -2, -1, 1, 2)]
test_ranges += [(-2 ** 63, 2 ** 63 - 2, 1)]
for start, end, step in test_ranges:
    iter1 = range(start, end, step)
    iter2 = pyrange(start, end, step)
    test_id = 'range({}, {}, {})'.format(start, end, step)
    assert_iterators_equal(iter1, iter2, test_id, limit=100)
    iter1 = reversed(range(start, end, step))
    iter2 = pyrange_reversed(start, end, step)
    test_id = 'reversed(range({}, {}, {}))'.format(start, end, step)
    assert_iterators_equal(iter1, iter2, test_id, limit=100)
print("RangeTest::test_range_iterators: ok")
