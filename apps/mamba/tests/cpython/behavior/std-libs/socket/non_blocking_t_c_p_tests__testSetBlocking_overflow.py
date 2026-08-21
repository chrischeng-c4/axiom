# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "non_blocking_t_c_p_tests__testSetBlocking_overflow"
# subject = "cpython.test_socket.NonBlockingTCPTests.testSetBlocking_overflow"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("NonBlockingTCPTests.testSetBlocking_overflow", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NonBlockingTCPTests.testSetBlocking_overflow did not pass"
print("NonBlockingTCPTests::testSetBlocking_overflow: ok")
