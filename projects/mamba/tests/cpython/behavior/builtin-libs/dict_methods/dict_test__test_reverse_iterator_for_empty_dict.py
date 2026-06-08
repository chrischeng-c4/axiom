# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_reverse_iterator_for_empty_dict"
# subject = "cpython.test_dict.DictTest.test_reverse_iterator_for_empty_dict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_reverse_iterator_for_empty_dict
"""Auto-ported test: DictTest::test_reverse_iterator_for_empty_dict (CPython 3.12 oracle)."""


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

assert list(reversed({})) == []

assert list(reversed({}.items())) == []

assert list(reversed({}.values())) == []

assert list(reversed({}.keys())) == []

assert list(reversed(dict())) == []

assert list(reversed(dict().items())) == []

assert list(reversed(dict().values())) == []

assert list(reversed(dict().keys())) == []
print("DictTest::test_reverse_iterator_for_empty_dict: ok")
