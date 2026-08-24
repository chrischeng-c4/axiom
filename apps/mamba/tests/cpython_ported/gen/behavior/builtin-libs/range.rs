use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/builtin-libs/range/range_test__test_empty.py`.
#[test]
fn test_gen_behavior_builtin_libs_range_range_test__test_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_empty"
# subject = "cpython.test_range.RangeTest.test_empty"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_empty
"""Auto-ported test: RangeTest::test_empty (CPython 3.12 oracle)."""


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
r = range(0)

assert 0 not in r

assert 1 not in r
r = range(0, -10)

assert 0 not in r

assert -1 not in r

assert 1 not in r
print("RangeTest::test_empty: ok")
"###);
    assert_output(&out, r###"RangeTest::test_empty: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/range/range_test__test_issue11845.py`.
#[test]
fn test_gen_behavior_builtin_libs_range_range_test__test_issue11845() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_issue11845"
# subject = "cpython.test_range.RangeTest.test_issue11845"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_issue11845
"""Auto-ported test: RangeTest::test_issue11845 (CPython 3.12 oracle)."""


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
r = range(*slice(1, 18, 2).indices(20))
values = {None, 0, 1, -1, 2, -2, 5, -5, 19, -19, 20, -20, 21, -21, 30, -30, 99, -99}
for i in values:
    for j in values:
        for k in values - {0}:
            r[i:j:k]
print("RangeTest::test_issue11845: ok")
"###);
    assert_output(&out, r###"RangeTest::test_issue11845: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/range/range_test__test_large_exhausted_iterator_pickling.py`.
#[test]
fn test_gen_behavior_builtin_libs_range_range_test__test_large_exhausted_iterator_pickling() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_large_exhausted_iterator_pickling"
# subject = "cpython.test_range.RangeTest.test_large_exhausted_iterator_pickling"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_large_exhausted_iterator_pickling
"""Auto-ported test: RangeTest::test_large_exhausted_iterator_pickling (CPython 3.12 oracle)."""


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
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    r = range(20)
    i = iter(r)
    while True:
        r = next(i)
        if r == 19:
            break
    d = pickle.dumps(i, proto)
    i2 = pickle.loads(d)

    assert list(i) == []

    assert list(i2) == []
print("RangeTest::test_large_exhausted_iterator_pickling: ok")
"###);
    assert_output(&out, r###"RangeTest::test_large_exhausted_iterator_pickling: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/range/range_test__test_pickling.py`.
#[test]
fn test_gen_behavior_builtin_libs_range_range_test__test_pickling() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_pickling"
# subject = "cpython.test_range.RangeTest.test_pickling"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_pickling
"""Auto-ported test: RangeTest::test_pickling (CPython 3.12 oracle)."""


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
testcases = [(13,), (0, 11), (-22, 10), (20, 3, -1), (13, 21, 3), (-2, 2, 2), (2 ** 65, 2 ** 65 + 2)]
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    for t in testcases:
        r = range(*t)

        assert list(pickle.loads(pickle.dumps(r, proto))) == list(r)
print("RangeTest::test_pickling: ok")
"###);
    assert_output(&out, r###"RangeTest::test_pickling: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/range/range_test__test_strided_limits.py`.
#[test]
fn test_gen_behavior_builtin_libs_range_range_test__test_strided_limits() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_strided_limits"
# subject = "cpython.test_range.RangeTest.test_strided_limits"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_strided_limits
"""Auto-ported test: RangeTest::test_strided_limits (CPython 3.12 oracle)."""


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
r = range(0, 101, 2)

assert 0 in r

assert 1 not in r

assert 2 in r

assert 99 not in r

assert 100 in r

assert 101 not in r
r = range(0, -20, -1)

assert 0 in r

assert -1 in r

assert -19 in r

assert -20 not in r
r = range(0, -20, -2)

assert -18 in r

assert -19 not in r

assert -20 not in r
print("RangeTest::test_strided_limits: ok")
"###);
    assert_output(&out, r###"RangeTest::test_strided_limits: ok
"###);
}
