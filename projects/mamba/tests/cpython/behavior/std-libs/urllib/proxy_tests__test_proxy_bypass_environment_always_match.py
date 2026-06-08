# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "proxy_tests__test_proxy_bypass_environment_always_match"
# subject = "cpython.test_urllib.ProxyTests.test_proxy_bypass_environment_always_match"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllib
_suite = unittest.defaultTestLoader.loadTestsFromName("ProxyTests.test_proxy_bypass_environment_always_match", test_urllib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ProxyTests.test_proxy_bypass_environment_always_match did not pass"
print("ProxyTests::test_proxy_bypass_environment_always_match: ok")
