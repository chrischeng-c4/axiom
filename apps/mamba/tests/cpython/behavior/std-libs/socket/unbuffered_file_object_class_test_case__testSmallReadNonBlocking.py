# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "unbuffered_file_object_class_test_case__testSmallReadNonBlocking"
# subject = "cpython.test_socket.UnbufferedFileObjectClassTestCase.testSmallReadNonBlocking"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("UnbufferedFileObjectClassTestCase.testSmallReadNonBlocking", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UnbufferedFileObjectClassTestCase.testSmallReadNonBlocking did not pass"
print("UnbufferedFileObjectClassTestCase::testSmallReadNonBlocking: ok")
