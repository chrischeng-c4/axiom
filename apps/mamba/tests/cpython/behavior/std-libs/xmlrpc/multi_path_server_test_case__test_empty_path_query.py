# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc"
# dimension = "behavior"
# case = "multi_path_server_test_case__test_empty_path_query"
# subject = "cpython.test_xmlrpc.MultiPathServerTestCase.test_empty_path_query"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xmlrpc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xmlrpc
_suite = unittest.defaultTestLoader.loadTestsFromName("MultiPathServerTestCase.test_empty_path_query", test_xmlrpc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MultiPathServerTestCase.test_empty_path_query did not pass"
print("MultiPathServerTestCase::test_empty_path_query: ok")
