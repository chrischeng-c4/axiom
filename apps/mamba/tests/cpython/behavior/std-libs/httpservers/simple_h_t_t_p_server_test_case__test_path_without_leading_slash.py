# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "httpservers"
# dimension = "behavior"
# case = "simple_h_t_t_p_server_test_case__test_path_without_leading_slash"
# subject = "cpython.test_httpservers.SimpleHTTPServerTestCase.test_path_without_leading_slash"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_httpservers
_suite = unittest.defaultTestLoader.loadTestsFromName("SimpleHTTPServerTestCase.test_path_without_leading_slash", test_httpservers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SimpleHTTPServerTestCase.test_path_without_leading_slash did not pass"
print("SimpleHTTPServerTestCase::test_path_without_leading_slash: ok")
