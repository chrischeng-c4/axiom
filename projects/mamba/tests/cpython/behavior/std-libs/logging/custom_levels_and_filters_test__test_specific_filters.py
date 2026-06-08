# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "custom_levels_and_filters_test__test_specific_filters"
# subject = "cpython.test_logging.CustomLevelsAndFiltersTest.test_specific_filters"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("CustomLevelsAndFiltersTest.test_specific_filters", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CustomLevelsAndFiltersTest.test_specific_filters did not pass"
print("CustomLevelsAndFiltersTest::test_specific_filters: ok")
