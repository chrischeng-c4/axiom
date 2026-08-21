# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "interrupted_send_timeout_test__testInterruptedSendmsgTimeout"
# subject = "cpython.test_socket.InterruptedSendTimeoutTest.testInterruptedSendmsgTimeout"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("InterruptedSendTimeoutTest.testInterruptedSendmsgTimeout", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython InterruptedSendTimeoutTest.testInterruptedSendmsgTimeout did not pass"
print("InterruptedSendTimeoutTest::testInterruptedSendmsgTimeout: ok")
