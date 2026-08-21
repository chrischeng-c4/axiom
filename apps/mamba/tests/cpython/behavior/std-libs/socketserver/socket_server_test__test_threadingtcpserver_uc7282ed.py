# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "behavior"
# case = "socket_server_test__test_threadingtcpserver_uc7282ed"
# subject = "cpython.test_socketserver.SocketServerTest.test_ThreadingTCPServer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socketserver.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socketserver
_suite = unittest.defaultTestLoader.loadTestsFromName("SocketServerTest.test_ThreadingTCPServer", test_socketserver)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SocketServerTest.test_ThreadingTCPServer did not pass"
print("SocketServerTest::test_ThreadingTCPServer: ok")
