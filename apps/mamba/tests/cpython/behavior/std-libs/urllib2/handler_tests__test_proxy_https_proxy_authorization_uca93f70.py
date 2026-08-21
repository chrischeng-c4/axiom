# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib2"
# dimension = "behavior"
# case = "handler_tests__test_proxy_https_proxy_authorization_uca93f70"
# subject = "cpython.test_urllib2.HandlerTests.test_proxy_https_proxy_authorization"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllib2
_suite = unittest.defaultTestLoader.loadTestsFromName("HandlerTests.test_proxy_https_proxy_authorization", test_urllib2)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython HandlerTests.test_proxy_https_proxy_authorization did not pass"
print("HandlerTests::test_proxy_https_proxy_authorization: ok")
