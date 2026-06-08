# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dictviews"
# dimension = "behavior"
# case = "dict_set_test__test_keys_set_operations"
# subject = "cpython.test_dictviews.DictSetTest.test_keys_set_operations"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dictviews.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dictviews.py::DictSetTest::test_keys_set_operations
"""Auto-ported test: DictSetTest::test_keys_set_operations (CPython 3.12 oracle)."""


import collections.abc
import copy
import pickle
import sys
import unittest
from test.support import C_RECURSION_LIMIT


# --- test body ---
d1 = {'a': 1, 'b': 2}
d2 = {'b': 3, 'c': 2}
d3 = {'d': 4, 'e': 5}
d4 = {'d': 4}

class CustomSet(set):

    def intersection(self, other):
        return CustomSet(super().intersection(other))

assert d1.keys() & d1.keys() == {'a', 'b'}

assert d1.keys() & d2.keys() == {'b'}

assert d1.keys() & d3.keys() == set()

assert d1.keys() & set(d1.keys()) == {'a', 'b'}

assert d1.keys() & set(d2.keys()) == {'b'}

assert d1.keys() & set(d3.keys()) == set()

assert d1.keys() & tuple(d1.keys()) == {'a', 'b'}

assert d3.keys() & d4.keys() == {'d'}

assert d4.keys() & d3.keys() == {'d'}

assert d4.keys() & set(d3.keys()) == {'d'}

assert isinstance(d4.keys() & frozenset(d3.keys()), set)

assert isinstance(frozenset(d3.keys()) & d4.keys(), set)

assert type(d4.keys() & CustomSet(d3.keys())) is set

assert type(d1.keys() & []) is set

assert type([] & d1.keys()) is set

assert d1.keys() | d1.keys() == {'a', 'b'}

assert d1.keys() | d2.keys() == {'a', 'b', 'c'}

assert d1.keys() | d3.keys() == {'a', 'b', 'd', 'e'}

assert d1.keys() | set(d1.keys()) == {'a', 'b'}

assert d1.keys() | set(d2.keys()) == {'a', 'b', 'c'}

assert d1.keys() | set(d3.keys()) == {'a', 'b', 'd', 'e'}

assert d1.keys() | (1, 2) == {'a', 'b', 1, 2}

assert d1.keys() ^ d1.keys() == set()

assert d1.keys() ^ d2.keys() == {'a', 'c'}

assert d1.keys() ^ d3.keys() == {'a', 'b', 'd', 'e'}

assert d1.keys() ^ set(d1.keys()) == set()

assert d1.keys() ^ set(d2.keys()) == {'a', 'c'}

assert d1.keys() ^ set(d3.keys()) == {'a', 'b', 'd', 'e'}

assert d1.keys() ^ tuple(d2.keys()) == {'a', 'c'}

assert d1.keys() - d1.keys() == set()

assert d1.keys() - d2.keys() == {'a'}

assert d1.keys() - d3.keys() == {'a', 'b'}

assert d1.keys() - set(d1.keys()) == set()

assert d1.keys() - set(d2.keys()) == {'a'}

assert d1.keys() - set(d3.keys()) == {'a', 'b'}

assert d1.keys() - (0, 1) == {'a', 'b'}

assert not d1.keys().isdisjoint(d1.keys())

assert not d1.keys().isdisjoint(d2.keys())

assert not d1.keys().isdisjoint(list(d2.keys()))

assert not d1.keys().isdisjoint(set(d2.keys()))

assert d1.keys().isdisjoint({'x', 'y', 'z'})

assert d1.keys().isdisjoint(['x', 'y', 'z'])

assert d1.keys().isdisjoint(set(['x', 'y', 'z']))

assert d1.keys().isdisjoint(set(['x', 'y']))

assert d1.keys().isdisjoint(['x', 'y'])

assert d1.keys().isdisjoint({})

assert d1.keys().isdisjoint(d3.keys())
de = {}

assert de.keys().isdisjoint(set())

assert de.keys().isdisjoint([])

assert de.keys().isdisjoint(de.keys())

assert de.keys().isdisjoint([1])
print("DictSetTest::test_keys_set_operations: ok")
