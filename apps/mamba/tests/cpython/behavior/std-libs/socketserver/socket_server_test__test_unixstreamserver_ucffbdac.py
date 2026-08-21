# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "behavior"
# case = "socket_server_test__test_unixstreamserver_ucffbdac"
# subject = "cpython.test_socketserver.SocketServerTest.test_UnixStreamServer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socketserver.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socketserver
_suite = unittest.defaultTestLoader.loadTestsFromName("SocketServerTest.test_UnixStreamServer", test_socketserver)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SocketServerTest.test_UnixStreamServer did not pass"
print("SocketServerTest::test_UnixStreamServer: ok")
