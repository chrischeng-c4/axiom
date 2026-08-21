# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "inheritance_test__test_SOCK_CLOEXEC"
# subject = "cpython.test_socket.InheritanceTest.test_SOCK_CLOEXEC"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("InheritanceTest.test_SOCK_CLOEXEC", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython InheritanceTest.test_SOCK_CLOEXEC did not pass"
print("InheritanceTest::test_SOCK_CLOEXEC: ok")
