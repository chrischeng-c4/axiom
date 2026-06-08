# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "httplib"
# dimension = "behavior"
# case = "source_address_test__testhttpsconnectionsourceaddress_uc07b057"
# subject = "cpython.test_httplib.SourceAddressTest.testHTTPSConnectionSourceAddress"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_httplib
_suite = unittest.defaultTestLoader.loadTestsFromName("SourceAddressTest.testHTTPSConnectionSourceAddress", test_httplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SourceAddressTest.testHTTPSConnectionSourceAddress did not pass"
print("SourceAddressTest::testHTTPSConnectionSourceAddress: ok")
