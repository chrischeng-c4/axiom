# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dictviews"
# dimension = "behavior"
# case = "dict_set_test__test_constructors_not_callable"
# subject = "cpython.test_dictviews.DictSetTest.test_constructors_not_callable"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dictviews.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dictviews.py::DictSetTest::test_constructors_not_callable
"""Auto-ported test: DictSetTest::test_constructors_not_callable (CPython 3.12 oracle)."""


import collections.abc
import copy
import pickle
import sys
import unittest
from test.support import C_RECURSION_LIMIT


# --- test body ---
kt = type({}.keys())

try:
    kt({})
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    kt()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
it = type({}.items())

try:
    it({})
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    it()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
vt = type({}.values())

try:
    vt({})
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    vt()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("DictSetTest::test_constructors_not_callable: ok")
