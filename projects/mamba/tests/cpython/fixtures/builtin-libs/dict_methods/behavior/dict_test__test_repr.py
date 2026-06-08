# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_repr"
# subject = "cpython.test_dict.DictTest.test_repr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_repr
"""Auto-ported test: DictTest::test_repr (CPython 3.12 oracle)."""


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

assert repr(d) == '{}'
d[1] = 2

assert repr(d) == '{1: 2}'
d = {}
d[1] = d

assert repr(d) == '{1: {...}}'

class Exc(Exception):
    pass

class BadRepr(object):

    def __repr__(self):
        raise Exc()
d = {1: BadRepr()}

try:
    repr(d)
    raise AssertionError('expected Exc')
except Exc:
    pass
print("DictTest::test_repr: ok")
