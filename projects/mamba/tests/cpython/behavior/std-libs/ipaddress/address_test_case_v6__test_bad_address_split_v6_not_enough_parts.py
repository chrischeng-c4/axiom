# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "address_test_case_v6__test_bad_address_split_v6_not_enough_parts"
# subject = "cpython.test_ipaddress.AddressTestCase_v6.test_bad_address_split_v6_not_enough_parts"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ipaddress
_suite = unittest.defaultTestLoader.loadTestsFromName("AddressTestCase_v6.test_bad_address_split_v6_not_enough_parts", test_ipaddress)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AddressTestCase_v6.test_bad_address_split_v6_not_enough_parts did not pass"
print("AddressTestCase_v6::test_bad_address_split_v6_not_enough_parts: ok")
