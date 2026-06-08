# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "defaultdict"
# dimension = "behavior"
# case = "test_default_dict__test_copy"
# subject = "cpython.test_defaultdict.TestDefaultDict.test_copy"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_defaultdict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_defaultdict.py::TestDefaultDict::test_copy
"""Auto-ported test: TestDefaultDict::test_copy (CPython 3.12 oracle)."""


import copy
import pickle
import unittest
from collections import defaultdict


'Unit tests for collections.defaultdict.'

def foobar():
    return list


# --- test body ---
d1 = defaultdict()
d2 = d1.copy()

assert type(d2) == defaultdict

assert d2.default_factory == None

assert d2 == {}
d1.default_factory = list
d3 = d1.copy()

assert type(d3) == defaultdict

assert d3.default_factory == list

assert d3 == {}
d1[42]
d4 = d1.copy()

assert type(d4) == defaultdict

assert d4.default_factory == list

assert d4 == {42: []}
d4[12]

assert d4 == {42: [], 12: []}
d = defaultdict()
d['a'] = 42
e = d.copy()

assert e['a'] == 42
print("TestDefaultDict::test_copy: ok")
