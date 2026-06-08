# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "augassign"
# dimension = "behavior"
# case = "aug_assign_test__test_in_list"
# subject = "cpython.test_augassign.AugAssignTest.testInList"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_augassign.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_augassign.py::AugAssignTest::testInList
"""Auto-ported test: AugAssignTest::testInList (CPython 3.12 oracle)."""


import unittest


# --- test body ---
x = [2]
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
print("AugAssignTest::testInList: ok")
