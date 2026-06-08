# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_container_iterator"
# subject = "cpython.test_dict.DictTest.test_container_iterator"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_container_iterator
"""Auto-ported test: DictTest::test_container_iterator (CPython 3.12 oracle)."""


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
class C(object):
    pass
views = (dict.items, dict.values, dict.keys)
for v in views:
    obj = C()
    ref = weakref.ref(obj)
    container = {obj: 1}
    obj.v = v(container)
    obj.x = iter(obj.v)
    del obj, container
    gc.collect()

    assert ref() is None
print("DictTest::test_container_iterator: ok")
