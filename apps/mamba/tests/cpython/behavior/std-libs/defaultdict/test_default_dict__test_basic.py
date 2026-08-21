# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "defaultdict"
# dimension = "behavior"
# case = "test_default_dict__test_basic"
# subject = "cpython.test_defaultdict.TestDefaultDict.test_basic"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_defaultdict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_defaultdict.py::TestDefaultDict::test_basic
"""Auto-ported test: TestDefaultDict::test_basic (CPython 3.12 oracle)."""


import copy
import pickle
import unittest
from collections import defaultdict


'Unit tests for collections.defaultdict.'

def foobar():
    return list


# --- test body ---
d1 = defaultdict()

assert d1.default_factory == None
d1.default_factory = list
d1[12].append(42)

assert d1 == {12: [42]}
d1[12].append(24)

assert d1 == {12: [42, 24]}
d1[13]
d1[14]

assert d1 == {12: [42, 24], 13: [], 14: []}

assert d1[12] is not d1[13] is not d1[14]
d2 = defaultdict(list, foo=1, bar=2)

assert d2.default_factory == list

assert d2 == {'foo': 1, 'bar': 2}

assert d2['foo'] == 1

assert d2['bar'] == 2

assert d2[42] == []

assert 'foo' in d2

assert 'foo' in d2.keys()

assert 'bar' in d2

assert 'bar' in d2.keys()

assert 42 in d2

assert 42 in d2.keys()

assert 12 not in d2

assert 12 not in d2.keys()
d2.default_factory = None

assert d2.default_factory == None
try:
    d2[15]
except KeyError as err:

    assert err.args == (15,)
else:

    raise AssertionError("d2[15] didn't raise KeyError")

try:
    defaultdict(1)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestDefaultDict::test_basic: ok")
