# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "behavior"
# case = "socket_server_test__test_threadingunixdatagramserver_uc0f876f"
# subject = "cpython.test_socketserver.SocketServerTest.test_ThreadingUnixDatagramServer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socketserver.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socketserver
_suite = unittest.defaultTestLoader.loadTestsFromName("SocketServerTest.test_ThreadingUnixDatagramServer", test_socketserver)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SocketServerTest.test_ThreadingUnixDatagramServer did not pass"
print("SocketServerTest::test_ThreadingUnixDatagramServer: ok")
