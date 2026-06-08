# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "httpservers"
# dimension = "behavior"
# case = "base_h_t_t_p_request_handler_test_case__test_close_connection"
# subject = "cpython.test_httpservers.BaseHTTPRequestHandlerTestCase.test_close_connection"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_httpservers
_suite = unittest.defaultTestLoader.loadTestsFromName("BaseHTTPRequestHandlerTestCase.test_close_connection", test_httpservers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BaseHTTPRequestHandlerTestCase.test_close_connection did not pass"
print("BaseHTTPRequestHandlerTestCase::test_close_connection: ok")
