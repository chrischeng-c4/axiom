# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_constructor"
# subject = "cpython.test_dict.DictTest.test_constructor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_constructor
"""Auto-ported test: DictTest::test_constructor (CPython 3.12 oracle)."""


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

assert dict() == {}

assert dict() is not {}
print("DictTest::test_constructor: ok")
