# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_reverse_iterator_for_shared_shared_dicts"
# subject = "cpython.test_dict.DictTest.test_reverse_iterator_for_shared_shared_dicts"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_reverse_iterator_for_shared_shared_dicts
"""Auto-ported test: DictTest::test_reverse_iterator_for_shared_shared_dicts (CPython 3.12 oracle)."""


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

    def __init__(self, x, y):
        if x:
            self.x = x
        if y:
            self.y = y

assert list(reversed(A(1, 2).__dict__)) == ['y', 'x']

assert list(reversed(A(1, 0).__dict__)) == ['x']

assert list(reversed(A(0, 1).__dict__)) == ['y']
print("DictTest::test_reverse_iterator_for_shared_shared_dicts: ok")
