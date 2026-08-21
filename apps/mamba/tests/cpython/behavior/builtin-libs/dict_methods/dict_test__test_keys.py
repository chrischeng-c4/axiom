# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_keys"
# subject = "cpython.test_dict.DictTest.test_keys"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_keys
"""Auto-ported test: DictTest::test_keys (CPython 3.12 oracle)."""


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

assert set(d.keys()) == set()
d = {'a': 1, 'b': 2}
k = d.keys()

assert set(k) == {'a', 'b'}

assert 'a' in k

assert 'b' in k

assert 'a' in d

assert 'b' in d

try:
    d.keys(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert repr(dict(a=1).keys()) == "dict_keys(['a'])"
print("DictTest::test_keys: ok")
