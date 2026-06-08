# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "defaultdict"
# dimension = "behavior"
# case = "test_default_dict__test_pickling"
# subject = "cpython.test_defaultdict.TestDefaultDict.test_pickling"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_defaultdict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_defaultdict.py::TestDefaultDict::test_pickling
"""Auto-ported test: TestDefaultDict::test_pickling (CPython 3.12 oracle)."""


import copy
import pickle
import unittest
from collections import defaultdict


'Unit tests for collections.defaultdict.'

def foobar():
    return list


# --- test body ---
d = defaultdict(int)
d[1]
for proto in range(pickle.HIGHEST_PROTOCOL + 1):
    s = pickle.dumps(d, proto)
    o = pickle.loads(s)

    assert d == o
print("TestDefaultDict::test_pickling: ok")
