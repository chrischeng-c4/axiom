# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "threaded_v_s_o_c_k_socket_stream_test__testStream"
# subject = "cpython.test_socket.ThreadedVSOCKSocketStreamTest.testStream"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("ThreadedVSOCKSocketStreamTest.testStream", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThreadedVSOCKSocketStreamTest.testStream did not pass"
print("ThreadedVSOCKSocketStreamTest::testStream: ok")
