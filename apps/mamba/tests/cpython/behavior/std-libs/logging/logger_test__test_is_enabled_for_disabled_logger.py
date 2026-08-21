# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "logger_test__test_is_enabled_for_disabled_logger"
# subject = "cpython.test_logging.LoggerTest.test_is_enabled_for_disabled_logger"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("LoggerTest.test_is_enabled_for_disabled_logger", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LoggerTest.test_is_enabled_for_disabled_logger did not pass"
print("LoggerTest::test_is_enabled_for_disabled_logger: ok")
