# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "create_server_functional_test__test_dual_stack_client_v6"
# subject = "cpython.test_socket.CreateServerFunctionalTest.test_dual_stack_client_v6"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("CreateServerFunctionalTest.test_dual_stack_client_v6", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CreateServerFunctionalTest.test_dual_stack_client_v6 did not pass"
print("CreateServerFunctionalTest::test_dual_stack_client_v6: ok")
