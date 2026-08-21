# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dictviews"
# dimension = "behavior"
# case = "dict_set_test__test_dict_items"
# subject = "cpython.test_dictviews.DictSetTest.test_dict_items"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dictviews.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dictviews.py::DictSetTest::test_dict_items
"""Auto-ported test: DictSetTest::test_dict_items (CPython 3.12 oracle)."""


import collections.abc
import copy
import pickle
import sys
import unittest
from test.support import C_RECURSION_LIMIT


# --- test body ---
d = {1: 10, 'a': 'ABC'}
items = d.items()

assert len(items) == 2

assert set(items) == {(1, 10), ('a', 'ABC')}

assert items == {(1, 10), ('a', 'ABC')}

assert items != {(1, 10), ('a', 'ABC'), 'junk'}

assert items != {(1, 10), ('a', 'def')}

assert items != {(1, 10)}

assert items != 42

assert (1, 10) in items

assert ('a', 'ABC') in items

assert (1, 11) not in items

assert 1 not in items

assert () not in items

assert (1,) not in items

assert (1, 2, 3) not in items

assert d.items() == d.items()
e = d.copy()

assert d.items() == e.items()
e['a'] = 'def'

assert d.items() != e.items()
print("DictSetTest::test_dict_items: ok")
