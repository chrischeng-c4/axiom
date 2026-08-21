# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "buffer_i_o_test__testRecvFromIntoSmallBuffer"
# subject = "cpython.test_socket.BufferIOTest.testRecvFromIntoSmallBuffer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("BufferIOTest.testRecvFromIntoSmallBuffer", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BufferIOTest.testRecvFromIntoSmallBuffer did not pass"
print("BufferIOTest::testRecvFromIntoSmallBuffer: ok")
