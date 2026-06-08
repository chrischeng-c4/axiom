# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllibnet"
# dimension = "behavior"
# case = "urlopen_network_tests__test_bad_address_ucdb1ee2"
# subject = "cpython.test_urllibnet.urlopenNetworkTests.test_bad_address"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllibnet.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllibnet
_suite = unittest.defaultTestLoader.loadTestsFromName("urlopenNetworkTests.test_bad_address", test_urllibnet)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython urlopenNetworkTests.test_bad_address did not pass"
print("urlopenNetworkTests::test_bad_address: ok")
