# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dictviews"
# dimension = "behavior"
# case = "dict_set_test__test_deeply_nested_repr"
# subject = "cpython.test_dictviews.DictSetTest.test_deeply_nested_repr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dictviews.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dictviews.py::DictSetTest::test_deeply_nested_repr
"""Auto-ported test: DictSetTest::test_deeply_nested_repr (CPython 3.12 oracle)."""


import collections.abc
import copy
import pickle
import sys
import unittest
from test.support import C_RECURSION_LIMIT


# --- test body ---
d = {}
for i in range(C_RECURSION_LIMIT // 2 + 100):
    d = {42: d.values()}

try:
    repr(d)
    raise AssertionError('expected RecursionError')
except RecursionError:
    pass
print("DictSetTest::test_deeply_nested_repr: ok")
