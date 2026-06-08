# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_simple_test__test_id_comparisons_uc3dfcaa"
# subject = "cpython.test_compare.ComparisonSimpleTest.test_id_comparisons"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_compare
_suite = unittest.defaultTestLoader.loadTestsFromName("ComparisonSimpleTest.test_id_comparisons", test_compare)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ComparisonSimpleTest.test_id_comparisons did not pass"
print("ComparisonSimpleTest::test_id_comparisons: ok")
