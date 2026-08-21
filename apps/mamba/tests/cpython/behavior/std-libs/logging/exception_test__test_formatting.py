# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "exception_test__test_formatting"
# subject = "cpython.test_logging.ExceptionTest.test_formatting"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("ExceptionTest.test_formatting", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ExceptionTest.test_formatting did not pass"
print("ExceptionTest::test_formatting: ok")
