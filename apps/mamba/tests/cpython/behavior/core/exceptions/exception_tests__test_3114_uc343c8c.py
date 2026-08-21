# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_3114_uc343c8c"
# subject = "cpython.test_exceptions.ExceptionTests.test_3114"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_exceptions
_suite = unittest.defaultTestLoader.loadTestsFromName("ExceptionTests.test_3114", test_exceptions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ExceptionTests.test_3114 did not pass"
print("ExceptionTests::test_3114: ok")
