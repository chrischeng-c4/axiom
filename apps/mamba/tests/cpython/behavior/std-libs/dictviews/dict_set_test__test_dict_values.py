# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dictviews"
# dimension = "behavior"
# case = "dict_set_test__test_dict_values"
# subject = "cpython.test_dictviews.DictSetTest.test_dict_values"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dictviews.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dictviews.py::DictSetTest::test_dict_values
"""Auto-ported test: DictSetTest::test_dict_values (CPython 3.12 oracle)."""


import collections.abc
import copy
import pickle
import sys
import unittest
from test.support import C_RECURSION_LIMIT


# --- test body ---
d = {1: 10, 'a': 'ABC'}
values = d.values()

assert set(values) == {10, 'ABC'}

assert len(values) == 2
print("DictSetTest::test_dict_values: ok")
