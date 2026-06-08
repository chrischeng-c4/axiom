# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "fraction_test__testBigComplexComparisons"
# subject = "cpython.test_fractions.FractionTest.testBigComplexComparisons"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_fractions
_suite = unittest.defaultTestLoader.loadTestsFromName("FractionTest.testBigComplexComparisons", test_fractions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FractionTest.testBigComplexComparisons did not pass"
print("FractionTest::testBigComplexComparisons: ok")
