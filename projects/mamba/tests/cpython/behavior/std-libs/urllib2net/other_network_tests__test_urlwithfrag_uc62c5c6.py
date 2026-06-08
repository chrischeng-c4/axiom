# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib2net"
# dimension = "behavior"
# case = "other_network_tests__test_urlwithfrag_uc62c5c6"
# subject = "cpython.test_urllib2net.OtherNetworkTests.test_urlwithfrag"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib2net.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllib2net
_suite = unittest.defaultTestLoader.loadTestsFromName("OtherNetworkTests.test_urlwithfrag", test_urllib2net)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython OtherNetworkTests.test_urlwithfrag did not pass"
print("OtherNetworkTests::test_urlwithfrag: ok")
