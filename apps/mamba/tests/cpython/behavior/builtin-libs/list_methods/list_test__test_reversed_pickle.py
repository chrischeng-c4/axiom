# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_reversed_pickle"
# subject = "cpython.test_list.ListTest.test_reversed_pickle"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_reversed_pickle
"""Auto-ported test: ListTest::test_reversed_pickle (CPython 3.12 oracle)."""


import sys
from test import list_tests
from test.support import cpython_only
import pickle
import unittest


# --- test body ---
type2test = list
orig = type2test([4, 5, 6, 7])
data = [10, 11, 12, 13, 14, 15]
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    itorig = reversed(orig)
    d = pickle.dumps((itorig, orig), proto)
    it, a = pickle.loads(d)
    a[:] = data

    assert type(it) == type(itorig)

    assert list(it) == data[len(orig) - 1::-1]
    next(itorig)
    d = pickle.dumps((itorig, orig), proto)
    it, a = pickle.loads(d)
    a[:] = data

    assert type(it) == type(itorig)

    assert list(it) == data[len(orig) - 2::-1]
    for i in range(1, len(orig)):
        next(itorig)
    d = pickle.dumps((itorig, orig), proto)
    it, a = pickle.loads(d)
    a[:] = data

    assert type(it) == type(itorig)

    assert list(it) == []

    try:
        next(itorig)
        raise AssertionError('expected StopIteration')
    except StopIteration:
        pass
    d = pickle.dumps((itorig, orig), proto)
    it, a = pickle.loads(d)
    a[:] = data

    assert list(it) == []
print("ListTest::test_reversed_pickle: ok")
