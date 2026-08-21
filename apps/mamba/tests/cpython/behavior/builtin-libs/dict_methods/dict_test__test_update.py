# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_update"
# subject = "cpython.test_dict.DictTest.test_update"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dict.py::DictTest::test_update
"""Auto-ported test: DictTest::test_update (CPython 3.12 oracle)."""


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
d.update({1: 100})
d.update({2: 20})
d.update({1: 1, 2: 2, 3: 3})

assert d == {1: 1, 2: 2, 3: 3}
d.update()

assert d == {1: 1, 2: 2, 3: 3}

try:
    d.update(None)
    raise AssertionError('expected (TypeError, AttributeError)')
except (TypeError, AttributeError):
    pass

class SimpleUserDict:

    def __init__(self):
        self.d = {1: 1, 2: 2, 3: 3}

    def keys(self):
        return self.d.keys()

    def __getitem__(self, i):
        return self.d[i]
d.clear()
d.update(SimpleUserDict())

assert d == {1: 1, 2: 2, 3: 3}

class Exc(Exception):
    pass
d.clear()

class FailingUserDict:

    def keys(self):
        raise Exc

try:
    d.update(FailingUserDict())
    raise AssertionError('expected Exc')
except Exc:
    pass

class FailingUserDict:

    def keys(self):

        class BogonIter:

            def __init__(self):
                self.i = 1

            def __iter__(self):
                return self

            def __next__(self):
                if self.i:
                    self.i = 0
                    return 'a'
                raise Exc
        return BogonIter()

    def __getitem__(self, key):
        return key

try:
    d.update(FailingUserDict())
    raise AssertionError('expected Exc')
except Exc:
    pass

class FailingUserDict:

    def keys(self):

        class BogonIter:

            def __init__(self):
                self.i = ord('a')

            def __iter__(self):
                return self

            def __next__(self):
                if self.i <= ord('z'):
                    rtn = chr(self.i)
                    self.i += 1
                    return rtn
                raise StopIteration
        return BogonIter()

    def __getitem__(self, key):
        raise Exc

try:
    d.update(FailingUserDict())
    raise AssertionError('expected Exc')
except Exc:
    pass

class badseq(object):

    def __iter__(self):
        return self

    def __next__(self):
        raise Exc()

try:
    {}.update(badseq())
    raise AssertionError('expected Exc')
except Exc:
    pass

try:
    {}.update([(1, 2, 3)])
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("DictTest::test_update: ok")
