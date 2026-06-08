# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_store_evilattr"
# subject = "cpython.test_dict.DictTest.test_store_evilattr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_store_evilattr
"""Auto-ported test: DictTest::test_store_evilattr (CPython 3.12 oracle)."""


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
class EvilAttr:

    def __init__(self, d):
        self.d = d

    def __del__(self):
        if 'attr' in self.d:
            del self.d['attr']
        gc.collect()

class Obj:
    pass
obj = Obj()
obj.__dict__ = {}
for _ in range(10):
    obj.attr = EvilAttr(obj.__dict__)
print("DictTest::test_store_evilattr: ok")
