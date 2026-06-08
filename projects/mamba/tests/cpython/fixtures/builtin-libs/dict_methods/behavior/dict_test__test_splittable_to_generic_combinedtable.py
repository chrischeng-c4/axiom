# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_splittable_to_generic_combinedtable"
# subject = "cpython.test_dict.DictTest.test_splittable_to_generic_combinedtable"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_splittable_to_generic_combinedtable
"""Auto-ported test: DictTest::test_splittable_to_generic_combinedtable (CPython 3.12 oracle)."""


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
"""split table must be correctly resized and converted to generic combined table"""

class C:
    pass
a = C()
a.x = 1
d = a.__dict__
before_resize = sys.getsizeof(d)
d[2] = 2

assert sys.getsizeof(d) > before_resize

assert list(d) == ['x', 2]
print("DictTest::test_splittable_to_generic_combinedtable: ok")
