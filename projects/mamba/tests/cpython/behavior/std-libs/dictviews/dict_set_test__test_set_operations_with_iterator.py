# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dictviews"
# dimension = "behavior"
# case = "dict_set_test__test_set_operations_with_iterator"
# subject = "cpython.test_dictviews.DictSetTest.test_set_operations_with_iterator"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dictviews.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dictviews.py::DictSetTest::test_set_operations_with_iterator
"""Auto-ported test: DictSetTest::test_set_operations_with_iterator (CPython 3.12 oracle)."""


import collections.abc
import copy
import pickle
import sys
import unittest
from test.support import C_RECURSION_LIMIT


# --- test body ---
origin = {1: 2, 3: 4}

assert origin.keys() & iter([1, 2]) == {1}

assert origin.keys() | iter([1, 2]) == {1, 2, 3}

assert origin.keys() ^ iter([1, 2]) == {2, 3}

assert origin.keys() - iter([1, 2]) == {3}
items = origin.items()

assert items & iter([(1, 2)]) == {(1, 2)}

assert items ^ iter([(1, 2)]) == {(3, 4)}

assert items | iter([(1, 2)]) == {(1, 2), (3, 4)}

assert items - iter([(1, 2)]) == {(3, 4)}
print("DictSetTest::test_set_operations_with_iterator: ok")
