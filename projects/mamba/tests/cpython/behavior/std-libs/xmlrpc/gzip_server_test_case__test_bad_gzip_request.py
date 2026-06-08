# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc"
# dimension = "behavior"
# case = "gzip_server_test_case__test_bad_gzip_request"
# subject = "cpython.test_xmlrpc.GzipServerTestCase.test_bad_gzip_request"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xmlrpc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xmlrpc
_suite = unittest.defaultTestLoader.loadTestsFromName("GzipServerTestCase.test_bad_gzip_request", test_xmlrpc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GzipServerTestCase.test_bad_gzip_request did not pass"
print("GzipServerTestCase::test_bad_gzip_request: ok")
