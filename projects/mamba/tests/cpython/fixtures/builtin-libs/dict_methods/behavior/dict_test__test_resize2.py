# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_resize2"
# subject = "cpython.test_dict.DictTest.test_resize2"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_resize2
"""Auto-ported test: DictTest::test_resize2 (CPython 3.12 oracle)."""


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
class X(object):

    def __hash__(self):
        return 5

    def __eq__(self, other):
        if resizing:
            d.clear()
        return False
d = {}
resizing = False
d[X()] = 1
d[X()] = 2
d[X()] = 3
d[X()] = 4
d[X()] = 5
resizing = True
d[9] = 6
print("DictTest::test_resize2: ok")
