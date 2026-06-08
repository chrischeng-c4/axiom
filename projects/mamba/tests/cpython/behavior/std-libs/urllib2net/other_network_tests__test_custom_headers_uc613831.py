# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib2net"
# dimension = "behavior"
# case = "other_network_tests__test_custom_headers_uc613831"
# subject = "cpython.test_urllib2net.OtherNetworkTests.test_custom_headers"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib2net.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllib2net
_suite = unittest.defaultTestLoader.loadTestsFromName("OtherNetworkTests.test_custom_headers", test_urllib2net)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython OtherNetworkTests.test_custom_headers did not pass"
print("OtherNetworkTests::test_custom_headers: ok")
