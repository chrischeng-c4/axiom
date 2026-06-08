# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_errors_in_view_containment_check"
# subject = "cpython.test_dict.DictTest.test_errors_in_view_containment_check"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_errors_in_view_containment_check
"""Auto-ported test: DictTest::test_errors_in_view_containment_check (CPython 3.12 oracle)."""


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
class C:

    def __eq__(self, other):
        raise RuntimeError
d1 = {1: C()}
d2 = {1: C()}
try:
    d1.items() == d2.items()
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
try:
    d1.items() != d2.items()
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
try:
    d1.items() <= d2.items()
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
try:
    d1.items() >= d2.items()
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
d3 = {1: C(), 2: C()}
try:
    d2.items() < d3.items()
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
try:
    d3.items() > d2.items()
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
print("DictTest::test_errors_in_view_containment_check: ok")
