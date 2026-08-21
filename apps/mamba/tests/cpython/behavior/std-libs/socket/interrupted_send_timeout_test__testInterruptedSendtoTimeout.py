# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "interrupted_send_timeout_test__testInterruptedSendtoTimeout"
# subject = "cpython.test_socket.InterruptedSendTimeoutTest.testInterruptedSendtoTimeout"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("InterruptedSendTimeoutTest.testInterruptedSendtoTimeout", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython InterruptedSendTimeoutTest.testInterruptedSendtoTimeout did not pass"
print("InterruptedSendTimeoutTest::testInterruptedSendtoTimeout: ok")
