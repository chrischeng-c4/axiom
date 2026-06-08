# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_missing"
# subject = "cpython.test_dict.DictTest.test_missing"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_missing
"""Auto-ported test: DictTest::test_missing (CPython 3.12 oracle)."""


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

assert not hasattr(dict, '__missing__')

assert not hasattr({}, '__missing__')

class D(dict):

    def __missing__(self, key):
        return 42
d = D({1: 2, 3: 4})

assert d[1] == 2

assert d[3] == 4

assert 2 not in d

assert 2 not in d.keys()

assert d[2] == 42

class E(dict):

    def __missing__(self, key):
        raise RuntimeError(key)
e = E()
try:
    e[42]
    raise AssertionError('expected RuntimeError')
except RuntimeError as _aR_e:
    import types as _types_aR
    c = _types_aR.SimpleNamespace(exception=_aR_e)

assert c.exception.args == (42,)

class F(dict):

    def __init__(self):
        self.__missing__ = lambda key: None
f = F()
try:
    f[42]
    raise AssertionError('expected KeyError')
except KeyError as _aR_e:
    import types as _types_aR
    c = _types_aR.SimpleNamespace(exception=_aR_e)

assert c.exception.args == (42,)

class G(dict):
    pass
g = G()
try:
    g[42]
    raise AssertionError('expected KeyError')
except KeyError as _aR_e:
    import types as _types_aR
    c = _types_aR.SimpleNamespace(exception=_aR_e)

assert c.exception.args == (42,)
print("DictTest::test_missing: ok")
