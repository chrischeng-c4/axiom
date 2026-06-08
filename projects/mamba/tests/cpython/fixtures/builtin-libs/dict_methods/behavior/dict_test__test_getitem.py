# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_getitem"
# subject = "cpython.test_dict.DictTest.test_getitem"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_getitem
"""Auto-ported test: DictTest::test_getitem (CPython 3.12 oracle)."""


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
d = {'a': 1, 'b': 2}

assert d['a'] == 1

assert d['b'] == 2
d['c'] = 3
d['a'] = 4

assert d['c'] == 3

assert d['a'] == 4
del d['b']

assert d == {'a': 4, 'c': 3}

try:
    d.__getitem__()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class BadEq(object):

    def __eq__(self, other):
        raise Exc()

    def __hash__(self):
        return 24
d = {}
d[BadEq()] = 42

try:
    d.__getitem__(23)
    raise AssertionError('expected KeyError')
except KeyError:
    pass

class Exc(Exception):
    pass

class BadHash(object):
    fail = False

    def __hash__(self):
        if self.fail:
            raise Exc()
        else:
            return 42
x = BadHash()
d[x] = 42
x.fail = True

try:
    d.__getitem__(x)
    raise AssertionError('expected Exc')
except Exc:
    pass
print("DictTest::test_getitem: ok")
