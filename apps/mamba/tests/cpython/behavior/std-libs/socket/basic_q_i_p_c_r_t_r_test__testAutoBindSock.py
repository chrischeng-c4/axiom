# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "basic_q_i_p_c_r_t_r_test__testAutoBindSock"
# subject = "cpython.test_socket.BasicQIPCRTRTest.testAutoBindSock"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("BasicQIPCRTRTest.testAutoBindSock", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BasicQIPCRTRTest.testAutoBindSock did not pass"
print("BasicQIPCRTRTest::testAutoBindSock: ok")
