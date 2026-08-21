# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib2"
# dimension = "behavior"
# case = "request_tests__test_url_fragment_ucaf8cae"
# subject = "cpython.test_urllib2.RequestTests.test_url_fragment"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllib2
_suite = unittest.defaultTestLoader.loadTestsFromName("RequestTests.test_url_fragment", test_urllib2)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RequestTests.test_url_fragment did not pass"
print("RequestTests::test_url_fragment: ok")
