# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "cookie_tests__test_expires_uc862598"
# subject = "cpython.test_http_cookiejar.CookieTests.test_expires"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_http_cookiejar
_suite = unittest.defaultTestLoader.loadTestsFromName("CookieTests.test_expires", test_http_cookiejar)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CookieTests.test_expires did not pass"
print("CookieTests::test_expires: ok")
