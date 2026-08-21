# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref"
# dimension = "behavior"
# case = "handler_tests__testenviron_uc5252bb"
# subject = "cpython.test_wsgiref.HandlerTests.testEnviron"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_wsgiref.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_wsgiref
_suite = unittest.defaultTestLoader.loadTestsFromName("HandlerTests.testEnviron", test_wsgiref)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython HandlerTests.testEnviron did not pass"
print("HandlerTests::testEnviron: ok")
