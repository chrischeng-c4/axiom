# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_deepcopy_weakvaluedict"
# subject = "cpython.test_copy.TestCopy.test_deepcopy_weakvaluedict"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_deepcopy_weakvaluedict
"""Auto-ported test: TestCopy::test_deepcopy_weakvaluedict (CPython 3.12 oracle)."""


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
class C(object):

    def __init__(self, i):
        self.i = i
a, b, c, d = [C(i) for i in range(4)]
u = weakref.WeakValueDictionary()
u[a] = b
u[c] = d
v = copy.deepcopy(u)

assert v != u

assert len(v) == 2
(x, y), (z, t) = sorted(v.items(), key=lambda pair: pair[0].i)

assert x is not a

assert x.i == a.i

assert y is b

assert z is not c

assert z.i == c.i

assert t is d
del x, y, z, t
del d
support.gc_collect()

assert len(v) == 1
print("TestCopy::test_deepcopy_weakvaluedict: ok")
