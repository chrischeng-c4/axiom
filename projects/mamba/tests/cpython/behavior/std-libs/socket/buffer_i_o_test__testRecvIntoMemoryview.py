# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "buffer_i_o_test__testRecvIntoMemoryview"
# subject = "cpython.test_socket.BufferIOTest.testRecvIntoMemoryview"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("BufferIOTest.testRecvIntoMemoryview", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BufferIOTest.testRecvIntoMemoryview did not pass"
print("BufferIOTest::testRecvIntoMemoryview: ok")
