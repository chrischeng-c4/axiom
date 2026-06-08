# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "httplib"
# dimension = "behavior"
# case = "timeout_test__testtimeoutattribute_ucc7aae0"
# subject = "cpython.test_httplib.TimeoutTest.testTimeoutAttribute"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_httplib
_suite = unittest.defaultTestLoader.loadTestsFromName("TimeoutTest.testTimeoutAttribute", test_httplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TimeoutTest.testTimeoutAttribute did not pass"
print("TimeoutTest::testTimeoutAttribute: ok")
