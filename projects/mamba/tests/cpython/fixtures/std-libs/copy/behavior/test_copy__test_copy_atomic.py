# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "test_copy__test_copy_atomic"
# subject = "cpython.test_copy.TestCopy.test_copy_atomic"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_copy.py::TestCopy::test_copy_atomic
"""Auto-ported test: TestCopy::test_copy_atomic (CPython 3.12 oracle)."""


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
class NewStyle:
    pass

def f():
    pass

class WithMetaclass(metaclass=abc.ABCMeta):
    pass
tests = [None, ..., NotImplemented, 42, 2 ** 100, 3.14, True, False, 1j, 'hello', 'helloሴ', f.__code__, b'world', bytes(range(256)), range(10), slice(1, 10, 2), NewStyle, max, WithMetaclass, property()]
for x in tests:

    assert copy.copy(x) is x
print("TestCopy::test_copy_atomic: ok")
