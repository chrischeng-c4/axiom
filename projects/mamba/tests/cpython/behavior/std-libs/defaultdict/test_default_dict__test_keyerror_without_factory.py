# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "defaultdict"
# dimension = "behavior"
# case = "test_default_dict__test_keyerror_without_factory"
# subject = "cpython.test_defaultdict.TestDefaultDict.test_keyerror_without_factory"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_defaultdict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_defaultdict.py::TestDefaultDict::test_keyerror_without_factory
"""Auto-ported test: TestDefaultDict::test_keyerror_without_factory (CPython 3.12 oracle)."""


import copy
import pickle
import unittest
from collections import defaultdict


'Unit tests for collections.defaultdict.'

def foobar():
    return list


# --- test body ---
d1 = defaultdict()
try:
    d1[1,]
except KeyError as err:

    assert err.args[0] == (1,)
else:

    raise AssertionError('expected KeyError')
print("TestDefaultDict::test_keyerror_without_factory: ok")
