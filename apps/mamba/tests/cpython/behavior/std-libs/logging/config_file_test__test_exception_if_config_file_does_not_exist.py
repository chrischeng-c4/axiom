# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "config_file_test__test_exception_if_config_file_does_not_exist"
# subject = "cpython.test_logging.ConfigFileTest.test_exception_if_config_file_does_not_exist"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("ConfigFileTest.test_exception_if_config_file_does_not_exist", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ConfigFileTest.test_exception_if_config_file_does_not_exist did not pass"
print("ConfigFileTest::test_exception_if_config_file_does_not_exist: ok")
