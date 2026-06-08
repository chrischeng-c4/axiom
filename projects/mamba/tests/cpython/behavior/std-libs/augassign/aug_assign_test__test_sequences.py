# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "augassign"
# dimension = "behavior"
# case = "aug_assign_test__test_sequences"
# subject = "cpython.test_augassign.AugAssignTest.testSequences"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_augassign.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_augassign.py::AugAssignTest::testSequences
"""Auto-ported test: AugAssignTest::testSequences (CPython 3.12 oracle)."""


import unittest


# --- test body ---
x = [1, 2]
x += [3, 4]
x *= 2

assert x == [1, 2, 3, 4, 1, 2, 3, 4]
x = [1, 2, 3]
y = x
x[1:2] *= 2
y[1:2] += [1]

assert x == [1, 2, 1, 2, 3]

assert x is y
print("AugAssignTest::testSequences: ok")
