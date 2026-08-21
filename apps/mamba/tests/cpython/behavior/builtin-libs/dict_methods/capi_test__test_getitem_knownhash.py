# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "capi_test__test_getitem_knownhash"
# subject = "cpython.test_dict.CAPITest.test_getitem_knownhash"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::CAPITest::test_getitem_knownhash
"""Auto-ported test: CAPITest::test_getitem_knownhash (CPython 3.12 oracle)."""


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
_testcapi = import_helper.import_module('_testcapi')
dict_getitem_knownhash = _testcapi.dict_getitem_knownhash
d = {'x': 1, 'y': 2, 'z': 3}

assert dict_getitem_knownhash(d, 'x', hash('x')) == 1

assert dict_getitem_knownhash(d, 'y', hash('y')) == 2

assert dict_getitem_knownhash(d, 'z', hash('z')) == 3

try:
    dict_getitem_knownhash([], 1, hash(1))
    raise AssertionError('expected SystemError')
except SystemError:
    pass

try:
    dict_getitem_knownhash({}, 1, hash(1))
    raise AssertionError('expected KeyError')
except KeyError:
    pass

class Exc(Exception):
    pass

class BadEq:

    def __eq__(self, other):
        raise Exc

    def __hash__(self):
        return 7
k1, k2 = (BadEq(), BadEq())
d = {k1: 1}

assert dict_getitem_knownhash(d, k1, hash(k1)) == 1

try:
    dict_getitem_knownhash(d, k2, hash(k2))
    raise AssertionError('expected Exc')
except Exc:
    pass
print("CAPITest::test_getitem_knownhash: ok")
