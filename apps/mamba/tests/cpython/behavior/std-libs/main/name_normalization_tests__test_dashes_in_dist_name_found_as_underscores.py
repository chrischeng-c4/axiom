# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "main"
# dimension = "behavior"
# case = "name_normalization_tests__test_dashes_in_dist_name_found_as_underscores"
# subject = "cpython.test_main.NameNormalizationTests.test_dashes_in_dist_name_found_as_underscores"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_main.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_main
_suite = unittest.defaultTestLoader.loadTestsFromName("NameNormalizationTests.test_dashes_in_dist_name_found_as_underscores", test_main)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NameNormalizationTests.test_dashes_in_dist_name_found_as_underscores did not pass"
print("NameNormalizationTests::test_dashes_in_dist_name_found_as_underscores: ok")
