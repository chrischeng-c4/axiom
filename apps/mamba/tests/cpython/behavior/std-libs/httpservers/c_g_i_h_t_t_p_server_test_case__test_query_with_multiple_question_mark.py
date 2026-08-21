# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "httpservers"
# dimension = "behavior"
# case = "c_g_i_h_t_t_p_server_test_case__test_query_with_multiple_question_mark"
# subject = "cpython.test_httpservers.CGIHTTPServerTestCase.test_query_with_multiple_question_mark"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_httpservers
_suite = unittest.defaultTestLoader.loadTestsFromName("CGIHTTPServerTestCase.test_query_with_multiple_question_mark", test_httpservers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CGIHTTPServerTestCase.test_query_with_multiple_question_mark did not pass"
print("CGIHTTPServerTestCase::test_query_with_multiple_question_mark: ok")
