# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_reentrant_insertion"
# subject = "cpython.test_dict.DictTest.test_reentrant_insertion"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_reentrant_insertion
"""Auto-ported test: DictTest::test_reentrant_insertion (CPython 3.12 oracle)."""


import collections
import collections.abc
import gc
import pickle
import random
import string
import sys
import unittest
import weakref
from test import support
from test.support import import_helper, C_RECURSION_LIMIT
from test import mapping_tests


class GeneralMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = dict

class Dict(dict):
    pass

class SubclassMappingTests(mapping_tests.BasicTestMappingProtocol):
    type2test = Dict


# --- test body ---
def _not_tracked(t):
    gc.collect()
    gc.collect()

    assert not gc.is_tracked(t)

def _tracked(t):

    assert gc.is_tracked(t)
    gc.collect()
    gc.collect()

    assert gc.is_tracked(t)

def check_reentrant_insertion(mutate):

    class Mutating:

        def __del__(self):
            mutate(d)
    d = {k: Mutating() for k in 'abcdefghijklmnopqr'}
    for k in list(d):
        d[k] = k

def helper_keys_contained(fn):
    empty = fn(dict())
    empty2 = fn(dict())
    smaller = fn({1: 1, 2: 2})
    larger = fn({1: 1, 2: 2, 3: 3})
    larger2 = fn({1: 1, 2: 2, 3: 3})
    larger3 = fn({4: 1, 2: 2, 3: 3})

    assert smaller < larger

    assert smaller <= larger

    assert larger > smaller

    assert larger >= smaller

    assert not smaller >= larger

    assert not smaller > larger

    assert not larger <= smaller

    assert not larger < smaller

    assert not smaller < larger3

    assert not smaller <= larger3

    assert not larger3 > smaller

    assert not larger3 >= smaller

    assert larger2 >= larger

    assert larger2 <= larger

    assert not larger2 > larger

    assert not larger2 < larger

    assert larger == larger2

    assert smaller != larger

    assert empty == empty2

    assert not empty != empty2

    assert not empty == smaller

    assert empty != smaller

    assert larger != larger3

    assert not larger == larger3

def make_shared_key_dict(n):

    class C:
        pass
    dicts = []
    for i in range(n):
        a = C()
        a.x, a.y, a.z = (1, 2, 3)
        dicts.append(a.__dict__)
    return dicts

def mutate(d):
    d['b'] = 5
check_reentrant_insertion(mutate)

def mutate(d):
    d.update(self.__dict__)
    d.clear()
check_reentrant_insertion(mutate)

def mutate(d):
    while d:
        d.popitem()
check_reentrant_insertion(mutate)
print("DictTest::test_reentrant_insertion: ok")
