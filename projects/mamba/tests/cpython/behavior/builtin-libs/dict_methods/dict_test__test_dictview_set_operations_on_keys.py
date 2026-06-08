# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_dictview_set_operations_on_keys"
# subject = "cpython.test_dict.DictTest.test_dictview_set_operations_on_keys"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_dictview_set_operations_on_keys
"""Auto-ported test: DictTest::test_dictview_set_operations_on_keys (CPython 3.12 oracle)."""


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
k1 = {1: 1, 2: 2}.keys()
k2 = {1: 1, 2: 2, 3: 3}.keys()
k3 = {4: 4}.keys()

assert k1 - k2 == set()

assert k1 - k3 == {1, 2}

assert k2 - k1 == {3}

assert k3 - k1 == {4}

assert k1 & k2 == {1, 2}

assert k1 & k3 == set()

assert k1 | k2 == {1, 2, 3}

assert k1 ^ k2 == {3}

assert k1 ^ k3 == {1, 2, 4}
print("DictTest::test_dictview_set_operations_on_keys: ok")
