# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_bad_key"
# subject = "cpython.test_dict.DictTest.test_bad_key"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_bad_key
"""Auto-ported test: DictTest::test_bad_key (CPython 3.12 oracle)."""


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
class CustomException(Exception):
    pass

class BadDictKey:

    def __hash__(self):
        return hash(self.__class__)

    def __eq__(self, other):
        if isinstance(other, self.__class__):
            raise CustomException
        return other
d = {}
x1 = BadDictKey()
x2 = BadDictKey()
d[x1] = 1
for stmt in ['d[x2] = 2', 'z = d[x2]', 'x2 in d', 'd.get(x2)', 'd.setdefault(x2, 42)', 'd.pop(x2)', 'd.update({x2: 2})']:
    try:
        exec(stmt, locals())
        raise AssertionError('expected CustomException')
    except CustomException:
        pass
print("DictTest::test_bad_key: ok")
