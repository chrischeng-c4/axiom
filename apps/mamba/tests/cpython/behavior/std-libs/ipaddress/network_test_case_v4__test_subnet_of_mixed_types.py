# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "network_test_case_v4__test_subnet_of_mixed_types"
# subject = "cpython.test_ipaddress.NetworkTestCase_v4.test_subnet_of_mixed_types"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ipaddress
_suite = unittest.defaultTestLoader.loadTestsFromName("NetworkTestCase_v4.test_subnet_of_mixed_types", test_ipaddress)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NetworkTestCase_v4.test_subnet_of_mixed_types did not pass"
print("NetworkTestCase_v4::test_subnet_of_mixed_types: ok")
