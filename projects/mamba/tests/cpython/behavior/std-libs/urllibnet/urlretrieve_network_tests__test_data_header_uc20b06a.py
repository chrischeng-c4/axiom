# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllibnet"
# dimension = "behavior"
# case = "urlretrieve_network_tests__test_data_header_uc20b06a"
# subject = "cpython.test_urllibnet.urlretrieveNetworkTests.test_data_header"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllibnet.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllibnet
_suite = unittest.defaultTestLoader.loadTestsFromName("urlretrieveNetworkTests.test_data_header", test_urllibnet)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython urlretrieveNetworkTests.test_data_header did not pass"
print("urlretrieveNetworkTests::test_data_header: ok")
