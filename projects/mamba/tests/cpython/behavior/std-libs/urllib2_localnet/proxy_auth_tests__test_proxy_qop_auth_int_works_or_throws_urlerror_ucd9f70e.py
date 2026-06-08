# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib2_localnet"
# dimension = "behavior"
# case = "proxy_auth_tests__test_proxy_qop_auth_int_works_or_throws_urlerror_ucd9f70e"
# subject = "cpython.test_urllib2_localnet.ProxyAuthTests.test_proxy_qop_auth_int_works_or_throws_urlerror"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib2_localnet.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllib2_localnet
_suite = unittest.defaultTestLoader.loadTestsFromName("ProxyAuthTests.test_proxy_qop_auth_int_works_or_throws_urlerror", test_urllib2_localnet)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ProxyAuthTests.test_proxy_qop_auth_int_works_or_throws_urlerror did not pass"
print("ProxyAuthTests::test_proxy_qop_auth_int_works_or_throws_urlerror: ok")
