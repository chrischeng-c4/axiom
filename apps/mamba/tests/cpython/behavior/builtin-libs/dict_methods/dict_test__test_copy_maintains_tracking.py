# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_copy_maintains_tracking"
# subject = "cpython.test_dict.DictTest.test_copy_maintains_tracking"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_copy_maintains_tracking
"""Auto-ported test: DictTest::test_copy_maintains_tracking (CPython 3.12 oracle)."""


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
class A:
    pass
key = A()
for d in ({}, {'a': 1}, {key: 'val'}):
    d2 = d.copy()

    assert gc.is_tracked(d) == gc.is_tracked(d2)
print("DictTest::test_copy_maintains_tracking: ok")
