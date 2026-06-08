# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "test_linux_abstract_namespace__testLinuxAbstractNamespace"
# subject = "cpython.test_socket.TestLinuxAbstractNamespace.testLinuxAbstractNamespace"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("TestLinuxAbstractNamespace.testLinuxAbstractNamespace", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestLinuxAbstractNamespace.testLinuxAbstractNamespace did not pass"
print("TestLinuxAbstractNamespace::testLinuxAbstractNamespace: ok")
