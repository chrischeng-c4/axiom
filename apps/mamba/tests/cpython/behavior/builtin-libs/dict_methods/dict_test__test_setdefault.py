# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_setdefault"
# subject = "cpython.test_dict.DictTest.test_setdefault"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_setdefault
"""Auto-ported test: DictTest::test_setdefault (CPython 3.12 oracle)."""


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

assert d.setdefault('key0') is None
d.setdefault('key0', [])

assert d.setdefault('key0') is None
d.setdefault('key', []).append(3)

assert d['key'][0] == 3
d.setdefault('key', []).append(4)

assert len(d['key']) == 2

try:
    d.setdefault()
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
    d.setdefault(x, [])
    raise AssertionError('expected Exc')
except Exc:
    pass
print("DictTest::test_setdefault: ok")
