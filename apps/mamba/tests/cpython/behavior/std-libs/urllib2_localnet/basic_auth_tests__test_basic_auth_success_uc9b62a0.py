# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib2_localnet"
# dimension = "behavior"
# case = "basic_auth_tests__test_basic_auth_success_uc9b62a0"
# subject = "cpython.test_urllib2_localnet.BasicAuthTests.test_basic_auth_success"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib2_localnet.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllib2_localnet
_suite = unittest.defaultTestLoader.loadTestsFromName("BasicAuthTests.test_basic_auth_success", test_urllib2_localnet)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BasicAuthTests.test_basic_auth_success did not pass"
print("BasicAuthTests::test_basic_auth_success: ok")
