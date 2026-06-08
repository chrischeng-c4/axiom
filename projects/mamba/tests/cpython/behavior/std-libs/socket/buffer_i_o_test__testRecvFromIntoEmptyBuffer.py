# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "buffer_i_o_test__testRecvFromIntoEmptyBuffer"
# subject = "cpython.test_socket.BufferIOTest.testRecvFromIntoEmptyBuffer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("BufferIOTest.testRecvFromIntoEmptyBuffer", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BufferIOTest.testRecvFromIntoEmptyBuffer did not pass"
print("BufferIOTest::testRecvFromIntoEmptyBuffer: ok")
