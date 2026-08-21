# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dictviews"
# dimension = "behavior"
# case = "dict_set_test__test_recursive_repr"
# subject = "cpython.test_dictviews.DictSetTest.test_recursive_repr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dictviews.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dictviews.py::DictSetTest::test_recursive_repr
"""Auto-ported test: DictSetTest::test_recursive_repr (CPython 3.12 oracle)."""


import collections.abc
import copy
import pickle
import sys
import unittest
from test.support import C_RECURSION_LIMIT


# --- test body ---
d = {}
d[42] = d.values()
r = repr(d)

assert isinstance(r, str)
d[42] = d.items()
r = repr(d)

assert isinstance(r, str)
print("DictSetTest::test_recursive_repr: ok")
