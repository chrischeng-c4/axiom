# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_dictview_mixed_set_operations"
# subject = "cpython.test_dict.DictTest.test_dictview_mixed_set_operations"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_dictview_mixed_set_operations
"""Auto-ported test: DictTest::test_dictview_mixed_set_operations (CPython 3.12 oracle)."""


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

assert {1: 1}.keys() == {1}

assert {1} == {1: 1}.keys()

assert {1: 1}.keys() | {2} == {1, 2}

assert {2} | {1: 1}.keys() == {1, 2}

assert {1: 1}.items() == {(1, 1)}

assert {(1, 1)} == {1: 1}.items()

assert {1: 1}.items() | {2} == {(1, 1), 2}

assert {2} | {1: 1}.items() == {(1, 1), 2}
print("DictTest::test_dictview_mixed_set_operations: ok")
