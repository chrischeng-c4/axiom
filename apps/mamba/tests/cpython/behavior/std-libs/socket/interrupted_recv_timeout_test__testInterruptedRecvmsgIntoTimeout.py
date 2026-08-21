# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "interrupted_recv_timeout_test__testInterruptedRecvmsgIntoTimeout"
# subject = "cpython.test_socket.InterruptedRecvTimeoutTest.testInterruptedRecvmsgIntoTimeout"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("InterruptedRecvTimeoutTest.testInterruptedRecvmsgIntoTimeout", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython InterruptedRecvTimeoutTest.testInterruptedRecvmsgIntoTimeout did not pass"
print("InterruptedRecvTimeoutTest::testInterruptedRecvmsgIntoTimeout: ok")
