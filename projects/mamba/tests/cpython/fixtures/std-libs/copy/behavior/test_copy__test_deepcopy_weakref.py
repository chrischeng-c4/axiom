# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_weakref"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_weakref"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_weakref
"""Auto-ported test: TestCopy::test_deepcopy_weakref (CPython 3.12 oracle)."""


import copy
import copyreg
import weakref
import abc
from operator import le, lt, ge, gt, eq, ne
import unittest
from test import support


'Unit tests for the copy module.'

order_comparisons = (le, lt, ge, gt)

equality_comparisons = (eq, ne)

comparisons = order_comparisons + equality_comparisons

def global_foo(x, y):
    return x + y


# --- test body ---
def _check_copy_weakdict(_dicttype):

    class C(object):
        pass
    a, b, c, d = [C() for i in range(4)]
    u = _dicttype()
    u[a] = b
    u[c] = d
    v = copy.copy(u)

    assert v is not u

    assert v == u

    assert v[a] == b

    assert v[c] == d

    assert len(v) == 2
    del c, d
    support.gc_collect()

    assert len(v) == 1
    x, y = (C(), C())
    v[x] = y

    assert x not in u

def _check_weakref(_copy):

    class C(object):
        pass
    obj = C()
    x = weakref.ref(obj)
    y = _copy(x)

    assert y is x
    del obj
    y = _copy(x)

    assert y is x
_check_weakref(copy.deepcopy)
print("TestCopy::test_deepcopy_weakref: ok")
