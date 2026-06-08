# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "defaultdict"
# dimension = "behavior"
# case = "test_default_dict__test_shallow_copy"
# subject = "cpython.test_defaultdict.TestDefaultDict.test_shallow_copy"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_defaultdict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_defaultdict.py::TestDefaultDict::test_shallow_copy
"""Auto-ported test: TestDefaultDict::test_shallow_copy (CPython 3.12 oracle)."""


import copy
import pickle
import unittest
from collections import defaultdict


'Unit tests for collections.defaultdict.'

def foobar():
    return list


# --- test body ---
d1 = defaultdict(foobar, {1: 1})
d2 = copy.copy(d1)

assert d2.default_factory == foobar

assert d2 == d1
d1.default_factory = list
d2 = copy.copy(d1)

assert d2.default_factory == list

assert d2 == d1
print("TestDefaultDict::test_shallow_copy: ok")
