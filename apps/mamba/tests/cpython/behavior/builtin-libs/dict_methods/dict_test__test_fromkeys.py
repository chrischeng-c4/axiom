# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_fromkeys"
# subject = "cpython.test_dict.DictTest.test_fromkeys"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_fromkeys
"""Auto-ported test: DictTest::test_fromkeys (CPython 3.12 oracle)."""


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

assert dict.fromkeys('abc') == {'a': None, 'b': None, 'c': None}
d = {}

assert d.fromkeys('abc') is not d

assert d.fromkeys('abc') == {'a': None, 'b': None, 'c': None}

assert d.fromkeys((4, 5), 0) == {4: 0, 5: 0}

assert d.fromkeys([]) == {}

def g():
    yield 1

assert d.fromkeys(g()) == {1: None}

try:
    {}.fromkeys(3)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class dictlike(dict):
    pass

assert dictlike.fromkeys('a') == {'a': None}

assert dictlike().fromkeys('a') == {'a': None}

assert isinstance(dictlike.fromkeys('a'), dictlike)

assert isinstance(dictlike().fromkeys('a'), dictlike)

class mydict(dict):

    def __new__(cls):
        return collections.UserDict()
ud = mydict.fromkeys('ab')

assert ud == {'a': None, 'b': None}

assert isinstance(ud, collections.UserDict)

try:
    dict.fromkeys()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class Exc(Exception):
    pass

class baddict1(dict):

    def __init__(self):
        raise Exc()

try:
    baddict1.fromkeys([1])
    raise AssertionError('expected Exc')
except Exc:
    pass

class BadSeq(object):

    def __iter__(self):
        return self

    def __next__(self):
        raise Exc()

try:
    dict.fromkeys(BadSeq())
    raise AssertionError('expected Exc')
except Exc:
    pass

class baddict2(dict):

    def __setitem__(self, key, value):
        raise Exc()

try:
    baddict2.fromkeys([1])
    raise AssertionError('expected Exc')
except Exc:
    pass
d = dict(zip(range(6), range(6)))

assert dict.fromkeys(d, 0) == dict(zip(range(6), [0] * 6))

class baddict3(dict):

    def __new__(cls):
        return d
d = {i: i for i in range(10)}
res = d.copy()
res.update(a=None, b=None, c=None)

assert baddict3.fromkeys({'a', 'b', 'c'}) == res
print("DictTest::test_fromkeys: ok")
