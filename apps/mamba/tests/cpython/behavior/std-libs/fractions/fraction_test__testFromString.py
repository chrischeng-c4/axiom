# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "behavior"
# case = "fraction_test__testFromString"
# subject = "cpython.test_fractions.FractionTest.testFromString"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_fractions
_suite = unittest.defaultTestLoader.loadTestsFromName("FractionTest.testFromString", test_fractions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FractionTest.testFromString did not pass"
print("FractionTest::testFromString: ok")
