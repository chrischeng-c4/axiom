# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "general_module_tests__test_socket_fileno_rejects_float"
# subject = "cpython.test_socket.GeneralModuleTests.test_socket_fileno_rejects_float"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("GeneralModuleTests.test_socket_fileno_rejects_float", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GeneralModuleTests.test_socket_fileno_rejects_float did not pass"
print("GeneralModuleTests::test_socket_fileno_rejects_float: ok")
