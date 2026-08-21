# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_pop"
# subject = "cpython.test_dict.DictTest.test_pop"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_pop
"""Auto-ported test: DictTest::test_pop (CPython 3.12 oracle)."""


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
d = {}
k, v = ('abc', 'def')
d[k] = v

try:
    d.pop('ghi')
    raise AssertionError('expected KeyError')
except KeyError:
    pass

assert d.pop(k) == v

assert len(d) == 0

try:
    d.pop(k)
    raise AssertionError('expected KeyError')
except KeyError:
    pass

assert d.pop(k, v) == v
d[k] = v

assert d.pop(k, 1) == v

try:
    d.pop()
    raise AssertionError('expected TypeError')
except TypeError:
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
    d.pop(x)
    raise AssertionError('expected Exc')
except Exc:
    pass
print("DictTest::test_pop: ok")
