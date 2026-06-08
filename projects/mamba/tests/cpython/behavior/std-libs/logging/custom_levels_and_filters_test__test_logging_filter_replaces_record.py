# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "custom_levels_and_filters_test__test_logging_filter_replaces_record"
# subject = "cpython.test_logging.CustomLevelsAndFiltersTest.test_logging_filter_replaces_record"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("CustomLevelsAndFiltersTest.test_logging_filter_replaces_record", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CustomLevelsAndFiltersTest.test_logging_filter_replaces_record did not pass"
print("CustomLevelsAndFiltersTest::test_logging_filter_replaces_record: ok")
