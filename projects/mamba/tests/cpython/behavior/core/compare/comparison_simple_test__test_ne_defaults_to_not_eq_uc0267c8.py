# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_simple_test__test_ne_defaults_to_not_eq_uc0267c8"
# subject = "cpython.test_compare.ComparisonSimpleTest.test_ne_defaults_to_not_eq"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_compare
_suite = unittest.defaultTestLoader.loadTestsFromName("ComparisonSimpleTest.test_ne_defaults_to_not_eq", test_compare)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ComparisonSimpleTest.test_ne_defaults_to_not_eq did not pass"
print("ComparisonSimpleTest::test_ne_defaults_to_not_eq: ok")
