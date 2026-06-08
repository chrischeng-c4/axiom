# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "future_test__test_names"
# subject = "cpython.test.test_future_stmt.test_future_flags.FutureTest.test_names"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future_flags.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_future_flags.py::FutureTest::test_names
"""Auto-ported test: FutureTest::test_names (CPython 3.12 oracle)."""


import unittest
import __future__


GOOD_SERIALS = ('alpha', 'beta', 'candidate', 'final')

features = __future__.all_feature_names


# --- test body ---
given_feature_names = features[:]
for name in dir(__future__):
    obj = getattr(__future__, name, None)
    if obj is not None and isinstance(obj, __future__._Feature):

        assert name in given_feature_names
        given_feature_names.remove(name)

assert len(given_feature_names) == 0
print("FutureTest::test_names: ok")
