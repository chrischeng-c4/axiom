# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urlparse"
# dimension = "behavior"
# case = "url_parse_test_case__test_rfc1808_uc1809db"
# subject = "cpython.test_urlparse.UrlParseTestCase.test_RFC1808"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urlparse
_suite = unittest.defaultTestLoader.loadTestsFromName("UrlParseTestCase.test_RFC1808", test_urlparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UrlParseTestCase.test_RFC1808 did not pass"
print("UrlParseTestCase::test_RFC1808: ok")
