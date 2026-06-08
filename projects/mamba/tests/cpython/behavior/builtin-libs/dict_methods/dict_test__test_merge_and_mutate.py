# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_merge_and_mutate"
# subject = "cpython.test_dict.DictTest.test_merge_and_mutate"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_merge_and_mutate
"""Auto-ported test: DictTest::test_merge_and_mutate (CPython 3.12 oracle)."""


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
class X:

    def __hash__(self):
        return 0

    def __eq__(self, o):
        other.clear()
        return False
l = [(i, 0) for i in range(1, 1337)]
other = dict(l)
other[X()] = 0
d = {X(): 0, 1: 1}

try:
    d.update(other)
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
print("DictTest::test_merge_and_mutate: ok")
