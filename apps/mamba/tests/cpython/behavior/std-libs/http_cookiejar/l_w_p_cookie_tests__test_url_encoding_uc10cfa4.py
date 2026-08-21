# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "l_w_p_cookie_tests__test_url_encoding_uc10cfa4"
# subject = "cpython.test_http_cookiejar.LWPCookieTests.test_url_encoding"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_http_cookiejar
_suite = unittest.defaultTestLoader.loadTestsFromName("LWPCookieTests.test_url_encoding", test_http_cookiejar)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LWPCookieTests.test_url_encoding did not pass"
print("LWPCookieTests::test_url_encoding: ok")
