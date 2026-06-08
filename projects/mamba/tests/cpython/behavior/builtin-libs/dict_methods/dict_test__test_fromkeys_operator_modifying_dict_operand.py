# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_fromkeys_operator_modifying_dict_operand"
# subject = "cpython.test_dict.DictTest.test_fromkeys_operator_modifying_dict_operand"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_fromkeys_operator_modifying_dict_operand
"""Auto-ported test: DictTest::test_fromkeys_operator_modifying_dict_operand (CPython 3.12 oracle)."""


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
class X(int):

    def __hash__(self):
        return 13

    def __eq__(self, other):
        if len(d) > 1:
            d.clear()
        return False
d = {}
d = {X(1): 1, X(2): 2}
try:
    dict.fromkeys(d)
except RuntimeError:
    pass
print("DictTest::test_fromkeys_operator_modifying_dict_operand: ok")
