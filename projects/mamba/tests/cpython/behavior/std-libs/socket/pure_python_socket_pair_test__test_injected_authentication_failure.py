# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "pure_python_socket_pair_test__test_injected_authentication_failure"
# subject = "cpython.test_socket.PurePythonSocketPairTest.test_injected_authentication_failure"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("PurePythonSocketPairTest.test_injected_authentication_failure", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PurePythonSocketPairTest.test_injected_authentication_failure did not pass"
print("PurePythonSocketPairTest::test_injected_authentication_failure: ok")
