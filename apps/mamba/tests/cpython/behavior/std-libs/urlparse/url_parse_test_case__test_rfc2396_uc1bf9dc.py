# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urlparse"
# dimension = "behavior"
# case = "url_parse_test_case__test_rfc2396_uc1bf9dc"
# subject = "cpython.test_urlparse.UrlParseTestCase.test_RFC2396"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urlparse
_suite = unittest.defaultTestLoader.loadTestsFromName("UrlParseTestCase.test_RFC2396", test_urlparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UrlParseTestCase.test_RFC2396 did not pass"
print("UrlParseTestCase::test_RFC2396: ok")
