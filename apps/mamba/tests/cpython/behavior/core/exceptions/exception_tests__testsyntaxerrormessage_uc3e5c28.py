# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__testsyntaxerrormessage_uc3e5c28"
# subject = "cpython.test_exceptions.ExceptionTests.testSyntaxErrorMessage"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_exceptions
_suite = unittest.defaultTestLoader.loadTestsFromName("ExceptionTests.testSyntaxErrorMessage", test_exceptions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ExceptionTests.testSyntaxErrorMessage did not pass"
print("ExceptionTests::testSyntaxErrorMessage: ok")
