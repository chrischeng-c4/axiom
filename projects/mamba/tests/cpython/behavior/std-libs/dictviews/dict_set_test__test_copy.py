# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dictviews"
# dimension = "behavior"
# case = "dict_set_test__test_copy"
# subject = "cpython.test_dictviews.DictSetTest.test_copy"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dictviews.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dictviews.py::DictSetTest::test_copy
"""Auto-ported test: DictSetTest::test_copy (CPython 3.12 oracle)."""


import collections.abc
import copy
import pickle
import sys
import unittest
from test.support import C_RECURSION_LIMIT


# --- test body ---
d = {1: 10, 'a': 'ABC'}

try:
    copy.copy(d.keys())
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    copy.copy(d.values())
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    copy.copy(d.items())
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("DictSetTest::test_copy: ok")
