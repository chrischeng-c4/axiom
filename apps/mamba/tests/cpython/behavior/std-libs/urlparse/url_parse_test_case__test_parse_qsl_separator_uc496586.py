# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urlparse"
# dimension = "behavior"
# case = "url_parse_test_case__test_parse_qsl_separator_uc496586"
# subject = "cpython.test_urlparse.UrlParseTestCase.test_parse_qsl_separator"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urlparse
_suite = unittest.defaultTestLoader.loadTestsFromName("UrlParseTestCase.test_parse_qsl_separator", test_urlparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UrlParseTestCase.test_parse_qsl_separator did not pass"
print("UrlParseTestCase::test_parse_qsl_separator: ok")
