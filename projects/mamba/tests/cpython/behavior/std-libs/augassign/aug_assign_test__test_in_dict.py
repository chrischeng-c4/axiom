# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "augassign"
# dimension = "behavior"
# case = "aug_assign_test__test_in_dict"
# subject = "cpython.test_augassign.AugAssignTest.testInDict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_augassign.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_augassign.py::AugAssignTest::testInDict
"""Auto-ported test: AugAssignTest::testInDict (CPython 3.12 oracle)."""


import unittest


# --- test body ---
x = {0: 2}
x[0] += 1
x[0] *= 2
x[0] **= 2
x[0] -= 8
x[0] //= 5
x[0] %= 3
x[0] &= 2
x[0] |= 5
x[0] ^= 1
x[0] /= 2

assert x[0] == 3.0
print("AugAssignTest::testInDict: ok")
