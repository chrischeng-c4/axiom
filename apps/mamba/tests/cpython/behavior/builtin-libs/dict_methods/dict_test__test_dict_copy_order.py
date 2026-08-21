# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_dict_copy_order"
# subject = "cpython.test_dict.DictTest.test_dict_copy_order"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_dict_copy_order
"""Auto-ported test: DictTest::test_dict_copy_order (CPython 3.12 oracle)."""


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
od = collections.OrderedDict([('a', 1), ('b', 2)])
od.move_to_end('a')
expected = list(od.items())
copy = dict(od)

assert list(copy.items()) == expected

class CustomDict(dict):
    pass
pairs = [('a', 1), ('b', 2), ('c', 3)]
d = CustomDict(pairs)

assert pairs == list(dict(d).items())

class CustomReversedDict(dict):

    def keys(self):
        return reversed(list(dict.keys(self)))
    __iter__ = keys

    def items(self):
        return reversed(dict.items(self))
d = CustomReversedDict(pairs)

assert pairs[::-1] == list(dict(d).items())
print("DictTest::test_dict_copy_order: ok")
