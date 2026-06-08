# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_full_test__test_str_subclass_ucdfb372"
# subject = "cpython.test_compare.ComparisonFullTest.test_str_subclass"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_compare
_suite = unittest.defaultTestLoader.loadTestsFromName("ComparisonFullTest.test_str_subclass", test_compare)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ComparisonFullTest.test_str_subclass did not pass"
print("ComparisonFullTest::test_str_subclass: ok")
