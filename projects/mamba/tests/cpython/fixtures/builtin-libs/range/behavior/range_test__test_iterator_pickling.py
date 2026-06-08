# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_iterator_pickling"
# subject = "cpython.test_range.RangeTest.test_iterator_pickling"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_iterator_pickling
"""Auto-ported test: RangeTest::test_iterator_pickling (CPython 3.12 oracle)."""


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
testcases = [(13,), (0, 11), (-22, 10), (20, 3, -1), (13, 21, 3), (-2, 2, 2)]
for M in (2 ** 31, 2 ** 63):
    testcases += [(M - 3, M - 1), (4 * M, 4 * M + 2), (M - 2, M - 1, 2), (-M + 1, -M, -2), (1, 2, M - 1), (-1, -2, -M), (1, M - 1, M - 1), (-1, -M, -M)]
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    for t in testcases:
        it = itorg = iter(range(*t))
        data = list(range(*t))
        d = pickle.dumps(it, proto)
        it = pickle.loads(d)

        assert type(itorg) == type(it)

        assert list(it) == data
        it = pickle.loads(d)
        try:
            next(it)
        except StopIteration:
            continue
        d = pickle.dumps(it, proto)
        it = pickle.loads(d)

        assert list(it) == data[1:]
print("RangeTest::test_iterator_pickling: ok")
