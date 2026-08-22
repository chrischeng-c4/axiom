use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/builtin-libs/slice/slice_test__test_copy.py`.
#[test]
fn test_gen_behavior_builtin_libs_slice_slice_test__test_copy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "slice"
# dimension = "behavior"
# case = "slice_test__test_copy"
# subject = "cpython.test_slice.SliceTest.test_copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_slice.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_slice.py::SliceTest::test_copy
"""Auto-ported test: SliceTest::test_copy (CPython 3.12 oracle)."""


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
s = slice(1, 10)
c = copy.copy(s)

assert s is c
s = slice(1, 10, 2)
c = copy.copy(s)

assert s is c
s = slice([1, 2], [3, 4], [5, 6])
c = copy.copy(s)

assert s is c

assert s.start is c.start

assert s.stop is c.stop

assert s.step is c.step
print("SliceTest::test_copy: ok")
"###);
    assert_output(&out, r###"SliceTest::test_copy: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/slice/slice_test__test_members.py`.
#[test]
fn test_gen_behavior_builtin_libs_slice_slice_test__test_members() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "slice"
# dimension = "behavior"
# case = "slice_test__test_members"
# subject = "cpython.test_slice.SliceTest.test_members"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_slice.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_slice.py::SliceTest::test_members
"""Auto-ported test: SliceTest::test_members (CPython 3.12 oracle)."""


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
s = slice(1)

assert s.start == None

assert s.stop == 1

assert s.step == None
s = slice(1, 2)

assert s.start == 1

assert s.stop == 2

assert s.step == None
s = slice(1, 2, 3)

assert s.start == 1

assert s.stop == 2

assert s.step == 3

class AnyClass:
    pass
obj = AnyClass()
s = slice(obj)

assert s.stop is obj
print("SliceTest::test_members: ok")
"###);
    assert_output(&out, r###"SliceTest::test_members: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/slice/slice_test__test_repr.py`.
#[test]
fn test_gen_behavior_builtin_libs_slice_slice_test__test_repr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "slice"
# dimension = "behavior"
# case = "slice_test__test_repr"
# subject = "cpython.test_slice.SliceTest.test_repr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_slice.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_slice.py::SliceTest::test_repr
"""Auto-ported test: SliceTest::test_repr (CPython 3.12 oracle)."""


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

assert repr(slice(1, 2, 3)) == 'slice(1, 2, 3)'
print("SliceTest::test_repr: ok")
"###);
    assert_output(&out, r###"SliceTest::test_repr: ok
"###);
}
