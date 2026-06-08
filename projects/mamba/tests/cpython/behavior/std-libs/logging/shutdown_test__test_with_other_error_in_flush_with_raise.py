# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "shutdown_test__test_with_other_error_in_flush_with_raise"
# subject = "cpython.test_logging.ShutdownTest.test_with_other_error_in_flush_with_raise"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("ShutdownTest.test_with_other_error_in_flush_with_raise", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ShutdownTest.test_with_other_error_in_flush_with_raise did not pass"
print("ShutdownTest::test_with_other_error_in_flush_with_raise: ok")
