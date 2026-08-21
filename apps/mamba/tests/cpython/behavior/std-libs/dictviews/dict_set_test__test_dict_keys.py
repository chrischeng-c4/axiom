# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dictviews"
# dimension = "behavior"
# case = "dict_set_test__test_dict_keys"
# subject = "cpython.test_dictviews.DictSetTest.test_dict_keys"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dictviews.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dictviews.py::DictSetTest::test_dict_keys
"""Auto-ported test: DictSetTest::test_dict_keys (CPython 3.12 oracle)."""


import collections.abc
import copy
import pickle
import sys
import unittest
from test.support import C_RECURSION_LIMIT


# --- test body ---
d = {1: 10, 'a': 'ABC'}
keys = d.keys()

assert len(keys) == 2

assert set(keys) == {1, 'a'}

assert keys == {1, 'a'}

assert keys != {1, 'a', 'b'}

assert keys != {1, 'b'}

assert keys != {1}

assert keys != 42

assert 1 in keys

assert 'a' in keys

assert 10 not in keys

assert 'Z' not in keys

assert d.keys() == d.keys()
e = {1: 11, 'a': 'def'}

assert d.keys() == e.keys()
del e['a']

assert d.keys() != e.keys()
print("DictSetTest::test_dict_keys: ok")
