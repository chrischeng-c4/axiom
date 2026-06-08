# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_track_dynamic"
# subject = "cpython.test_dict.DictTest.test_track_dynamic"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_track_dynamic
"""Auto-ported test: DictTest::test_track_dynamic (CPython 3.12 oracle)."""


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

class MyObject(object):
    pass
x, y, z, w, o = (1.5, 'a', (1, object()), [], MyObject())
d = dict()
_not_tracked(d)
d[1] = 'a'
_not_tracked(d)
d[y] = 2
_not_tracked(d)
d[z] = 3
_not_tracked(d)
_not_tracked(d.copy())
d[4] = w
_tracked(d)
_tracked(d.copy())
d[4] = None
_not_tracked(d)
_not_tracked(d.copy())
d = dict()
dd = dict()
d[1] = dd
_not_tracked(dd)
_tracked(d)
dd[1] = d
_tracked(dd)
d = dict.fromkeys([x, y, z])
_not_tracked(d)
dd = dict()
dd.update(d)
_not_tracked(dd)
d = dict.fromkeys([x, y, z, o])
_tracked(d)
dd = dict()
dd.update(d)
_tracked(dd)
d = dict(x=x, y=y, z=z)
_not_tracked(d)
d = dict(x=x, y=y, z=z, w=w)
_tracked(d)
d = dict()
d.update(x=x, y=y, z=z)
_not_tracked(d)
d.update(w=w)
_tracked(d)
d = dict([(x, y), (z, 1)])
_not_tracked(d)
d = dict([(x, y), (z, w)])
_tracked(d)
d = dict()
d.update([(x, y), (z, 1)])
_not_tracked(d)
d.update([(x, y), (z, w)])
_tracked(d)
print("DictTest::test_track_dynamic: ok")
