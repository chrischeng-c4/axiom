# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "logger_adapter_test__test_find_caller_with_stacklevel"
# subject = "cpython.test_logging.LoggerAdapterTest.test_find_caller_with_stacklevel"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("LoggerAdapterTest.test_find_caller_with_stacklevel", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LoggerAdapterTest.test_find_caller_with_stacklevel did not pass"
print("LoggerAdapterTest::test_find_caller_with_stacklevel: ok")
