# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_iterator_pickling"
# subject = "cpython.test_dict.DictTest.test_iterator_pickling"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_iterator_pickling
"""Auto-ported test: DictTest::test_iterator_pickling (CPython 3.12 oracle)."""


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
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    data = {1: 'a', 2: 'b', 3: 'c'}
    it = iter(data)
    d = pickle.dumps(it, proto)
    it = pickle.loads(d)

    assert list(it) == list(data)
    it = pickle.loads(d)
    try:
        drop = next(it)
    except StopIteration:
        continue
    d = pickle.dumps(it, proto)
    it = pickle.loads(d)
    del data[drop]

    assert list(it) == list(data)
print("DictTest::test_iterator_pickling: ok")
