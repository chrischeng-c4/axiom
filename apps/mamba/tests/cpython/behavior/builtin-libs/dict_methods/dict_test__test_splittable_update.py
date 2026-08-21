# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_splittable_update"
# subject = "cpython.test_dict.DictTest.test_splittable_update"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_splittable_update
"""Auto-ported test: DictTest::test_splittable_update (CPython 3.12 oracle)."""


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
"""dict.update(other) must preserve order in other."""

class C:

    def __init__(self, order):
        if order:
            self.a, self.b, self.c = (1, 2, 3)
        else:
            self.c, self.b, self.a = (1, 2, 3)
o = C(True)
o = C(False)

assert list(o.__dict__) == ['c', 'b', 'a']
d = {}
d.update(o.__dict__)

assert list(d) == ['c', 'b', 'a']
print("DictTest::test_splittable_update: ok")
