# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "defaultdict"
# dimension = "behavior"
# case = "test_default_dict__test_repr"
# subject = "cpython.test_defaultdict.TestDefaultDict.test_repr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_defaultdict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_defaultdict.py::TestDefaultDict::test_repr
"""Auto-ported test: TestDefaultDict::test_repr (CPython 3.12 oracle)."""


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

assert repr(d1) == 'defaultdict(None, {})'

assert eval(repr(d1)) == d1
d1[11] = 41

assert repr(d1) == 'defaultdict(None, {11: 41})'
d2 = defaultdict(int)

assert d2.default_factory == int
d2[12] = 42

assert repr(d2) == "defaultdict(<class 'int'>, {12: 42})"

def foo():
    return 43
d3 = defaultdict(foo)

assert d3.default_factory is foo
d3[13]

assert repr(d3) == 'defaultdict(%s, {13: 43})' % repr(foo)
print("TestDefaultDict::test_repr: ok")
