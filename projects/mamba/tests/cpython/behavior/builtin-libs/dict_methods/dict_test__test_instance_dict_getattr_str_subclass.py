# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_instance_dict_getattr_str_subclass"
# subject = "cpython.test_dict.DictTest.test_instance_dict_getattr_str_subclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_instance_dict_getattr_str_subclass
"""Auto-ported test: DictTest::test_instance_dict_getattr_str_subclass (CPython 3.12 oracle)."""


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
class Foo:

    def __init__(self, msg):
        self.msg = msg
f = Foo('123')

class _str(str):
    pass

assert f.msg == getattr(f, _str('msg'))

assert f.msg == f.__dict__[_str('msg')]
print("DictTest::test_instance_dict_getattr_str_subclass: ok")
