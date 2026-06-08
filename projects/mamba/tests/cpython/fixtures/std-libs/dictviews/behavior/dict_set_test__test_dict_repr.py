# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dictviews"
# dimension = "behavior"
# case = "dict_set_test__test_dict_repr"
# subject = "cpython.test_dictviews.DictSetTest.test_dict_repr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dictviews.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dictviews.py::DictSetTest::test_dict_repr
"""Auto-ported test: DictSetTest::test_dict_repr (CPython 3.12 oracle)."""


import collections.abc
import copy
import pickle
import sys
import unittest
from test.support import C_RECURSION_LIMIT


# --- test body ---
d = {1: 10, 'a': 'ABC'}

assert isinstance(repr(d), str)
r = repr(d.items())

assert isinstance(r, str)

assert r == "dict_items([('a', 'ABC'), (1, 10)])" or r == "dict_items([(1, 10), ('a', 'ABC')])"
r = repr(d.keys())

assert isinstance(r, str)

assert r == "dict_keys(['a', 1])" or r == "dict_keys([1, 'a'])"
r = repr(d.values())

assert isinstance(r, str)

assert r == "dict_values(['ABC', 10])" or r == "dict_values([10, 'ABC'])"
print("DictSetTest::test_dict_repr: ok")
