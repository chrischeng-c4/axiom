# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "nonblock_constant_test__test_SOCK_NONBLOCK"
# subject = "cpython.test_socket.NonblockConstantTest.test_SOCK_NONBLOCK"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("NonblockConstantTest.test_SOCK_NONBLOCK", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NonblockConstantTest.test_SOCK_NONBLOCK did not pass"
print("NonblockConstantTest::test_SOCK_NONBLOCK: ok")
