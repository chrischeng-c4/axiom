# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib2"
# dimension = "behavior"
# case = "misc_tests__test_parse_proxy_ucd4f991"
# subject = "cpython.test_urllib2.MiscTests.test_parse_proxy"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllib2
_suite = unittest.defaultTestLoader.loadTestsFromName("MiscTests.test_parse_proxy", test_urllib2)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MiscTests.test_parse_proxy did not pass"
print("MiscTests::test_parse_proxy: ok")
