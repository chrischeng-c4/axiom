# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc"
# dimension = "behavior"
# case = "simple_server_encoding_test_case__test_server_encoding"
# subject = "cpython.test_xmlrpc.SimpleServerEncodingTestCase.test_server_encoding"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xmlrpc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xmlrpc
_suite = unittest.defaultTestLoader.loadTestsFromName("SimpleServerEncodingTestCase.test_server_encoding", test_xmlrpc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SimpleServerEncodingTestCase.test_server_encoding did not pass"
print("SimpleServerEncodingTestCase::test_server_encoding: ok")
