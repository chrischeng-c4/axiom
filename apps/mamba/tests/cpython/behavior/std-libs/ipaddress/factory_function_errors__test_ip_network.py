# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "behavior"
# case = "factory_function_errors__test_ip_network"
# subject = "cpython.test_ipaddress.FactoryFunctionErrors.test_ip_network"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ipaddress.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ipaddress
_suite = unittest.defaultTestLoader.loadTestsFromName("FactoryFunctionErrors.test_ip_network", test_ipaddress)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FactoryFunctionErrors.test_ip_network did not pass"
print("FactoryFunctionErrors::test_ip_network: ok")
