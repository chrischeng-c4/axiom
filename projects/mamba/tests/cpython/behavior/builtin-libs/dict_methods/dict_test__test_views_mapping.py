# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_views_mapping"
# subject = "cpython.test_dict.DictTest.test_views_mapping"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_views_mapping
"""Auto-ported test: DictTest::test_views_mapping (CPython 3.12 oracle)."""


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
mappingproxy = type(type.__dict__)

class Dict(dict):
    pass
for cls in [dict, Dict]:
    d = cls()
    m1 = d.keys().mapping
    m2 = d.values().mapping
    m3 = d.items().mapping
    for m in [m1, m2, m3]:

        assert isinstance(m, mappingproxy)

        assert m == d
    d['foo'] = 'bar'
    for m in [m1, m2, m3]:

        assert isinstance(m, mappingproxy)

        assert m == d
print("DictTest::test_views_mapping: ok")
