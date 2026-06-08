# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllibnet"
# dimension = "behavior"
# case = "urlretrieve_network_tests__test_reporthook_ucd6e0c1"
# subject = "cpython.test_urllibnet.urlretrieveNetworkTests.test_reporthook"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllibnet.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllibnet
_suite = unittest.defaultTestLoader.loadTestsFromName("urlretrieveNetworkTests.test_reporthook", test_urllibnet)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython urlretrieveNetworkTests.test_reporthook did not pass"
print("urlretrieveNetworkTests::test_reporthook: ok")
