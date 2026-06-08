# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "general_module_tests__testIPv4_inet_aton_fourbytes"
# subject = "cpython.test_socket.GeneralModuleTests.testIPv4_inet_aton_fourbytes"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("GeneralModuleTests.testIPv4_inet_aton_fourbytes", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GeneralModuleTests.testIPv4_inet_aton_fourbytes did not pass"
print("GeneralModuleTests::testIPv4_inet_aton_fourbytes: ok")
