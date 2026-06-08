# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dictviews"
# dimension = "behavior"
# case = "dict_set_test__test_items_set_operations"
# subject = "cpython.test_dictviews.DictSetTest.test_items_set_operations"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dictviews.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dictviews.py::DictSetTest::test_items_set_operations
"""Auto-ported test: DictSetTest::test_items_set_operations (CPython 3.12 oracle)."""


import collections.abc
import copy
import pickle
import sys
import unittest
from test.support import C_RECURSION_LIMIT


# --- test body ---
d1 = {'a': 1, 'b': 2}
d2 = {'a': 2, 'b': 2}
d3 = {'d': 4, 'e': 5}

assert d1.items() & d1.items() == {('a', 1), ('b', 2)}

assert d1.items() & d2.items() == {('b', 2)}

assert d1.items() & d3.items() == set()

assert d1.items() & set(d1.items()) == {('a', 1), ('b', 2)}

assert d1.items() & set(d2.items()) == {('b', 2)}

assert d1.items() & set(d3.items()) == set()

assert d1.items() & (('a', 1), ('b', 2)) == {('a', 1), ('b', 2)}

assert d1.items() & (('a', 2), ('b', 2)) == {('b', 2)}

assert d1.items() & (('d', 4), ('e', 5)) == set()

assert d1.items() | d1.items() == {('a', 1), ('b', 2)}

assert d1.items() | d2.items() == {('a', 1), ('a', 2), ('b', 2)}

assert d1.items() | d3.items() == {('a', 1), ('b', 2), ('d', 4), ('e', 5)}

assert d1.items() | set(d1.items()) == {('a', 1), ('b', 2)}

assert d1.items() | set(d2.items()) == {('a', 1), ('a', 2), ('b', 2)}

assert d1.items() | set(d3.items()) == {('a', 1), ('b', 2), ('d', 4), ('e', 5)}

assert d1.items() | (('a', 1), ('b', 2)) == {('a', 1), ('b', 2)}

assert d1.items() | (('a', 2), ('b', 2)) == {('a', 1), ('a', 2), ('b', 2)}

assert d1.items() | (('d', 4), ('e', 5)) == {('a', 1), ('b', 2), ('d', 4), ('e', 5)}

assert d1.items() ^ d1.items() == set()

assert d1.items() ^ d2.items() == {('a', 1), ('a', 2)}

assert d1.items() ^ d3.items() == {('a', 1), ('b', 2), ('d', 4), ('e', 5)}

assert d1.items() ^ (('a', 1), ('b', 2)) == set()

assert d1.items() ^ (('a', 2), ('b', 2)) == {('a', 1), ('a', 2)}

assert d1.items() ^ (('d', 4), ('e', 5)) == {('a', 1), ('b', 2), ('d', 4), ('e', 5)}

assert d1.items() - d1.items() == set()

assert d1.items() - d2.items() == {('a', 1)}

assert d1.items() - d3.items() == {('a', 1), ('b', 2)}

assert d1.items() - set(d1.items()) == set()

assert d1.items() - set(d2.items()) == {('a', 1)}

assert d1.items() - set(d3.items()) == {('a', 1), ('b', 2)}

assert d1.items() - (('a', 1), ('b', 2)) == set()

assert d1.items() - (('a', 2), ('b', 2)) == {('a', 1)}

assert d1.items() - (('d', 4), ('e', 5)) == {('a', 1), ('b', 2)}

assert not d1.items().isdisjoint(d1.items())

assert not d1.items().isdisjoint(d2.items())

assert not d1.items().isdisjoint(list(d2.items()))

assert not d1.items().isdisjoint(set(d2.items()))

assert d1.items().isdisjoint({'x', 'y', 'z'})

assert d1.items().isdisjoint(['x', 'y', 'z'])

assert d1.items().isdisjoint(set(['x', 'y', 'z']))

assert d1.items().isdisjoint(set(['x', 'y']))

assert d1.items().isdisjoint({})

assert d1.items().isdisjoint(d3.items())
de = {}

assert de.items().isdisjoint(set())

assert de.items().isdisjoint([])

assert de.items().isdisjoint(de.items())

assert de.items().isdisjoint([1])
print("DictSetTest::test_items_set_operations: ok")
