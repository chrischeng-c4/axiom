# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_copy"
# subject = "cpython.test_dict.DictTest.test_copy"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_copy
"""Auto-ported test: DictTest::test_copy (CPython 3.12 oracle)."""


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
d = {1: 1, 2: 2, 3: 3}

assert d.copy() is not d

assert d.copy() == d

assert d.copy() == {1: 1, 2: 2, 3: 3}
copy = d.copy()
d[4] = 4

assert copy != d

assert {}.copy() == {}

try:
    d.copy(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("DictTest::test_copy: ok")
