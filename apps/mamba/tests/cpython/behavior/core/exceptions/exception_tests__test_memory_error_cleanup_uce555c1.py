# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_memory_error_cleanup_uce555c1"
# subject = "cpython.test_exceptions.ExceptionTests.test_memory_error_cleanup"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_exceptions
_suite = unittest.defaultTestLoader.loadTestsFromName("ExceptionTests.test_memory_error_cleanup", test_exceptions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ExceptionTests.test_memory_error_cleanup did not pass"
print("ExceptionTests::test_memory_error_cleanup: ok")
