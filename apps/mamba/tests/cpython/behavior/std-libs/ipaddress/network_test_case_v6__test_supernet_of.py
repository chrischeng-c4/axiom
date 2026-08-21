# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "network_test_case_v6__test_supernet_of"
# subject = "cpython.test_ipaddress.NetworkTestCase_v6.test_supernet_of"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ipaddress
_suite = unittest.defaultTestLoader.loadTestsFromName("NetworkTestCase_v6.test_supernet_of", test_ipaddress)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NetworkTestCase_v6.test_supernet_of did not pass"
print("NetworkTestCase_v6::test_supernet_of: ok")
