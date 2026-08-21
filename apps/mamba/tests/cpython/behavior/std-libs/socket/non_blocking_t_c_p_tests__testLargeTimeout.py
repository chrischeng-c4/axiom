# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "non_blocking_t_c_p_tests__testLargeTimeout"
# subject = "cpython.test_socket.NonBlockingTCPTests.testLargeTimeout"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("NonBlockingTCPTests.testLargeTimeout", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NonBlockingTCPTests.testLargeTimeout did not pass"
print("NonBlockingTCPTests::testLargeTimeout: ok")
