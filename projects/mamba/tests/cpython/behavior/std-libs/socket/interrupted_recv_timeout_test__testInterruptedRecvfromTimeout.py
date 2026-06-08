# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "interrupted_recv_timeout_test__testInterruptedRecvfromTimeout"
# subject = "cpython.test_socket.InterruptedRecvTimeoutTest.testInterruptedRecvfromTimeout"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("InterruptedRecvTimeoutTest.testInterruptedRecvfromTimeout", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython InterruptedRecvTimeoutTest.testInterruptedRecvfromTimeout did not pass"
print("InterruptedRecvTimeoutTest::testInterruptedRecvfromTimeout: ok")
