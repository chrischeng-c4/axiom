# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dictviews"
# dimension = "behavior"
# case = "dict_set_test__test_pickle"
# subject = "cpython.test_dictviews.DictSetTest.test_pickle"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dictviews.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dictviews.py::DictSetTest::test_pickle
"""Auto-ported test: DictSetTest::test_pickle (CPython 3.12 oracle)."""


import collections.abc
import copy
import pickle
import sys
import unittest
from test.support import C_RECURSION_LIMIT


# --- test body ---
d = {1: 10, 'a': 'ABC'}
for proto in range(pickle.HIGHEST_PROTOCOL + 1):

    try:
        pickle.dumps(d.keys(), proto)
        raise AssertionError('expected (TypeError, pickle.PicklingError)')
    except (TypeError, pickle.PicklingError):
        pass

    try:
        pickle.dumps(d.values(), proto)
        raise AssertionError('expected (TypeError, pickle.PicklingError)')
    except (TypeError, pickle.PicklingError):
        pass

    try:
        pickle.dumps(d.items(), proto)
        raise AssertionError('expected (TypeError, pickle.PicklingError)')
    except (TypeError, pickle.PicklingError):
        pass
print("DictSetTest::test_pickle: ok")
