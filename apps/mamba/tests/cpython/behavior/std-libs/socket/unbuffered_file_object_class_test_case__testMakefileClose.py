# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "unbuffered_file_object_class_test_case__testMakefileClose"
# subject = "cpython.test_socket.UnbufferedFileObjectClassTestCase.testMakefileClose"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socket
_suite = unittest.defaultTestLoader.loadTestsFromName("UnbufferedFileObjectClassTestCase.testMakefileClose", test_socket)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UnbufferedFileObjectClassTestCase.testMakefileClose did not pass"
print("UnbufferedFileObjectClassTestCase::testMakefileClose: ok")
