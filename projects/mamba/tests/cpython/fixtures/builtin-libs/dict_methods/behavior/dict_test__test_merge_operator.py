# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_merge_operator"
# subject = "cpython.test_dict.DictTest.test_merge_operator"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_merge_operator
"""Auto-ported test: DictTest::test_merge_operator (CPython 3.12 oracle)."""


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
a = {0: 0, 1: 1, 2: 1}
b = {1: 1, 2: 2, 3: 3}
c = a.copy()
c |= b

assert a | b == {0: 0, 1: 1, 2: 2, 3: 3}

assert c == {0: 0, 1: 1, 2: 2, 3: 3}
c = b.copy()
c |= a

assert b | a == {1: 1, 2: 1, 3: 3, 0: 0}

assert c == {1: 1, 2: 1, 3: 3, 0: 0}
c = a.copy()
c |= [(1, 1), (2, 2), (3, 3)]

assert c == {0: 0, 1: 1, 2: 2, 3: 3}

assert a.__or__(None) is NotImplemented

assert a.__or__(()) is NotImplemented

assert a.__or__('BAD') is NotImplemented

assert a.__or__('') is NotImplemented

try:
    a.__ior__(None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert a.__ior__(()) == {0: 0, 1: 1, 2: 1}

try:
    a.__ior__('BAD')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert a.__ior__('') == {0: 0, 1: 1, 2: 1}
print("DictTest::test_merge_operator: ok")
