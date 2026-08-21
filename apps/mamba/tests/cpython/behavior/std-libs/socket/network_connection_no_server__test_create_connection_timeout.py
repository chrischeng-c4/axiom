# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "network_connection_no_server__test_create_connection_timeout"
# subject = "cpython.test_socket.NetworkConnectionNoServer.test_create_connection_timeout"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("NetworkConnectionNoServer.test_create_connection_timeout", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NetworkConnectionNoServer.test_create_connection_timeout did not pass"
print("NetworkConnectionNoServer::test_create_connection_timeout: ok")
