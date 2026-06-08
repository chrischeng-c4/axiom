# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "main"
# dimension = "behavior"
# case = "name_normalization_tests__test_dist_name_found_as_any_case"
# subject = "cpython.test_main.NameNormalizationTests.test_dist_name_found_as_any_case"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_main.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_main
_suite = unittest.defaultTestLoader.loadTestsFromName("NameNormalizationTests.test_dist_name_found_as_any_case", test_main)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NameNormalizationTests.test_dist_name_found_as_any_case did not pass"
print("NameNormalizationTests::test_dist_name_found_as_any_case: ok")
