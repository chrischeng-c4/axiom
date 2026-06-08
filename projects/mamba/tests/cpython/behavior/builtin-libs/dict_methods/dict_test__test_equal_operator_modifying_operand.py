# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_equal_operator_modifying_operand"
# subject = "cpython.test_dict.DictTest.test_equal_operator_modifying_operand"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_equal_operator_modifying_operand
"""Auto-ported test: DictTest::test_equal_operator_modifying_operand (CPython 3.12 oracle)."""


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
class X:

    def __del__(self):
        dict_b.clear()

    def __eq__(self, other):
        dict_a.clear()
        return True

    def __hash__(self):
        return 13
dict_a = {X(): 0}
dict_b = {X(): X()}

assert dict_a == dict_b

class Y:

    def __eq__(self, other):
        dict_d.clear()
        return True
dict_c = {0: Y()}
dict_d = {0: set()}

assert dict_c == dict_d
print("DictTest::test_equal_operator_modifying_operand: ok")
