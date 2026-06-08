# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_empty_presized_dict_in_freelist"
# subject = "cpython.test_dict.DictTest.test_empty_presized_dict_in_freelist"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_empty_presized_dict_in_freelist
"""Auto-ported test: DictTest::test_empty_presized_dict_in_freelist (CPython 3.12 oracle)."""


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
try:
    d = {'a': 1 // 0, 'b': None, 'c': None, 'd': None, 'e': None, 'f': None, 'g': None, 'h': None}
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass
d = {}
print("DictTest::test_empty_presized_dict_in_freelist: ok")
