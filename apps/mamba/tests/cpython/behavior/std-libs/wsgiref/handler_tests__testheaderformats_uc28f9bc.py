# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref"
# dimension = "behavior"
# case = "handler_tests__testheaderformats_uc28f9bc"
# subject = "cpython.test_wsgiref.HandlerTests.testHeaderFormats"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_wsgiref.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_wsgiref
_suite = unittest.defaultTestLoader.loadTestsFromName("HandlerTests.testHeaderFormats", test_wsgiref)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython HandlerTests.testHeaderFormats did not pass"
print("HandlerTests::testHeaderFormats: ok")
