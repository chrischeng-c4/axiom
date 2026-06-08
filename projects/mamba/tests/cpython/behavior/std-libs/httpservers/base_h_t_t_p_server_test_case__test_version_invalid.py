# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "httpservers"
# dimension = "behavior"
# case = "base_h_t_t_p_server_test_case__test_version_invalid"
# subject = "cpython.test_httpservers.BaseHTTPServerTestCase.test_version_invalid"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_httpservers
_suite = unittest.defaultTestLoader.loadTestsFromName("BaseHTTPServerTestCase.test_version_invalid", test_httpservers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BaseHTTPServerTestCase.test_version_invalid did not pass"
print("BaseHTTPServerTestCase::test_version_invalid: ok")
