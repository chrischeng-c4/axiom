# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "non_blocking_t_c_p_tests__testInheritFlagsBlocking"
# subject = "cpython.test_socket.NonBlockingTCPTests.testInheritFlagsBlocking"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("NonBlockingTCPTests.testInheritFlagsBlocking", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NonBlockingTCPTests.testInheritFlagsBlocking did not pass"
print("NonBlockingTCPTests::testInheritFlagsBlocking: ok")
