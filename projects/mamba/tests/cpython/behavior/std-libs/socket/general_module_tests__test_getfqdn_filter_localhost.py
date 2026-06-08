# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "general_module_tests__test_getfqdn_filter_localhost"
# subject = "cpython.test_socket.GeneralModuleTests.test_getfqdn_filter_localhost"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("GeneralModuleTests.test_getfqdn_filter_localhost", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GeneralModuleTests.test_getfqdn_filter_localhost did not pass"
print("GeneralModuleTests::test_getfqdn_filter_localhost: ok")
