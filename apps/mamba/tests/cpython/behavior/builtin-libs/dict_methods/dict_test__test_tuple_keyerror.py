# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_tuple_keyerror"
# subject = "cpython.test_dict.DictTest.test_tuple_keyerror"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_tuple_keyerror
"""Auto-ported test: DictTest::test_tuple_keyerror (CPython 3.12 oracle)."""


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
d = {}
try:
    d[1,]
    raise AssertionError('expected KeyError')
except KeyError as _aR_e:
    import types as _types_aR
    c = _types_aR.SimpleNamespace(exception=_aR_e)

assert c.exception.args == ((1,),)
print("DictTest::test_tuple_keyerror: ok")
