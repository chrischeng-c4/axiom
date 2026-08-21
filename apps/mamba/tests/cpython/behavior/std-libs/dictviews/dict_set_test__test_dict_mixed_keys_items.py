# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dictviews"
# dimension = "behavior"
# case = "dict_set_test__test_dict_mixed_keys_items"
# subject = "cpython.test_dictviews.DictSetTest.test_dict_mixed_keys_items"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dictviews.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dictviews.py::DictSetTest::test_dict_mixed_keys_items
"""Auto-ported test: DictSetTest::test_dict_mixed_keys_items (CPython 3.12 oracle)."""


import collections.abc
import copy
import pickle
import sys
import unittest
from test.support import C_RECURSION_LIMIT


# --- test body ---
d = {(1, 1): 11, (2, 2): 22}
e = {1: 1, 2: 2}

assert d.keys() == e.items()

assert d.items() != e.keys()
print("DictSetTest::test_dict_mixed_keys_items: ok")
