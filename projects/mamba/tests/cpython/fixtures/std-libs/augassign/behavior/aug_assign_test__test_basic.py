# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "augassign"
# dimension = "behavior"
# case = "aug_assign_test__test_basic"
# subject = "cpython.test_augassign.AugAssignTest.testBasic"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_augassign.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_augassign.py::AugAssignTest::testBasic
"""Auto-ported test: AugAssignTest::testBasic (CPython 3.12 oracle)."""


import unittest


# --- test body ---
x = 2
x += 1
x *= 2
x **= 2
x -= 8
x //= 5
x %= 3
x &= 2
x |= 5
x ^= 1
x /= 2

assert x == 3.0
print("AugAssignTest::testBasic: ok")
