# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib2_localnet"
# dimension = "behavior"
# case = "proxy_auth_tests__test_proxy_with_bad_password_raises_httperror_ucefb453"
# subject = "cpython.test_urllib2_localnet.ProxyAuthTests.test_proxy_with_bad_password_raises_httperror"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib2_localnet.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllib2_localnet
_suite = unittest.defaultTestLoader.loadTestsFromName("ProxyAuthTests.test_proxy_with_bad_password_raises_httperror", test_urllib2_localnet)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ProxyAuthTests.test_proxy_with_bad_password_raises_httperror did not pass"
print("ProxyAuthTests::test_proxy_with_bad_password_raises_httperror: ok")
