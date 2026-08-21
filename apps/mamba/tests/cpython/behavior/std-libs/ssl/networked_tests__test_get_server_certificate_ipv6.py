# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "networked_tests__test_get_server_certificate_ipv6"
# subject = "cpython.test_ssl.NetworkedTests.test_get_server_certificate_ipv6"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ssl
_suite = unittest.defaultTestLoader.loadTestsFromName("NetworkedTests.test_get_server_certificate_ipv6", test_ssl)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NetworkedTests.test_get_server_certificate_ipv6 did not pass"
print("NetworkedTests::test_get_server_certificate_ipv6: ok")
