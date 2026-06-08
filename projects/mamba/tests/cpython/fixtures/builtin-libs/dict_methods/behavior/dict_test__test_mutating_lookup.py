# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_mutating_lookup"
# subject = "cpython.test_dict.DictTest.test_mutating_lookup"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_mutating_lookup
"""Auto-ported test: DictTest::test_mutating_lookup (CPython 3.12 oracle)."""


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
class NastyKey:
    mutate_dict = None

    def __init__(self, value):
        self.value = value

    def __hash__(self):
        return 1

    def __eq__(self, other):
        if NastyKey.mutate_dict:
            mydict, key = NastyKey.mutate_dict
            NastyKey.mutate_dict = None
            del mydict[key]
        return self.value == other.value
key1 = NastyKey(1)
key2 = NastyKey(2)
d = {key1: 1}
NastyKey.mutate_dict = (d, key1)
d[key2] = 2

assert d == {key2: 2}
print("DictTest::test_mutating_lookup: ok")
