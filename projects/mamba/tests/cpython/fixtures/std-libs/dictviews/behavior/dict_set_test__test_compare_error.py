# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dictviews"
# dimension = "behavior"
# case = "dict_set_test__test_compare_error"
# subject = "cpython.test_dictviews.DictSetTest.test_compare_error"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dictviews.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dictviews.py::DictSetTest::test_compare_error
"""Auto-ported test: DictSetTest::test_compare_error (CPython 3.12 oracle)."""


import collections.abc
import copy
import pickle
import sys
import unittest
from test.support import C_RECURSION_LIMIT


# --- test body ---
class Exc(Exception):
    pass

class BadEq:

    def __hash__(self):
        return 7

    def __eq__(self, other):
        raise Exc
k1, k2 = (BadEq(), BadEq())
v1, v2 = (BadEq(), BadEq())
d = {k1: v1}

assert k1 in d

assert k1 in d.keys()

assert v1 in d.values()

assert (k1, v1) in d.items()

try:
    d.__contains__(k2)
    raise AssertionError('expected Exc')
except Exc:
    pass

try:
    d.keys().__contains__(k2)
    raise AssertionError('expected Exc')
except Exc:
    pass

try:
    d.items().__contains__((k2, v1))
    raise AssertionError('expected Exc')
except Exc:
    pass

try:
    d.items().__contains__((k1, v2))
    raise AssertionError('expected Exc')
except Exc:
    pass
try:
    v2 in d.values()
    raise AssertionError('expected Exc')
except Exc:
    pass
print("DictSetTest::test_compare_error: ok")
