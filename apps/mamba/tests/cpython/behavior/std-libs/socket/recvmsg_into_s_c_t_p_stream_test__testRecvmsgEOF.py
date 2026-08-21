# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "recvmsg_into_s_c_t_p_stream_test__testRecvmsgEOF"
# subject = "cpython.test_socket.RecvmsgIntoSCTPStreamTest.testRecvmsgEOF"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("RecvmsgIntoSCTPStreamTest.testRecvmsgEOF", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RecvmsgIntoSCTPStreamTest.testRecvmsgEOF did not pass"
print("RecvmsgIntoSCTPStreamTest::testRecvmsgEOF: ok")
