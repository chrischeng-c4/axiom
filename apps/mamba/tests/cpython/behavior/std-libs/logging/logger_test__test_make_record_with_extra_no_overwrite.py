# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "logger_test__test_make_record_with_extra_no_overwrite"
# subject = "cpython.test_logging.LoggerTest.test_make_record_with_extra_no_overwrite"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("LoggerTest.test_make_record_with_extra_no_overwrite", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LoggerTest.test_make_record_with_extra_no_overwrite did not pass"
print("LoggerTest::test_make_record_with_extra_no_overwrite: ok")
