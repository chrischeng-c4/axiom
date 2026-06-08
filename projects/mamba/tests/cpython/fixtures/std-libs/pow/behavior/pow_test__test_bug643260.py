# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pow"
# dimension = "behavior"
# case = "pow_test__test_bug643260"
# subject = "cpython.test_pow.PowTest.test_bug643260"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pow.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pow.py::PowTest::test_bug643260
"""Auto-ported test: PowTest::test_bug643260 (CPython 3.12 oracle)."""


import math
import unittest


# --- test body ---
class TestRpow:

    def __rpow__(self, other):
        return None
None ** TestRpow()
print("PowTest::test_bug643260: ok")
