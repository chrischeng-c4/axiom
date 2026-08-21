# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgi"
# dimension = "behavior"
# case = "cgi_tests__testqsandurlencode_uc5a8b05"
# subject = "cpython.test_cgi.CgiTests.testQSAndUrlEncode"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cgi.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_cgi
_suite = unittest.defaultTestLoader.loadTestsFromName("CgiTests.testQSAndUrlEncode", test_cgi)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CgiTests.testQSAndUrlEncode did not pass"
print("CgiTests::testQSAndUrlEncode: ok")
