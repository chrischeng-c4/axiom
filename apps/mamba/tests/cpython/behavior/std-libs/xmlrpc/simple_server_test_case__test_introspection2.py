# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc"
# dimension = "behavior"
# case = "simple_server_test_case__test_introspection2"
# subject = "cpython.test_xmlrpc.SimpleServerTestCase.test_introspection2"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xmlrpc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xmlrpc
_suite = unittest.defaultTestLoader.loadTestsFromName("SimpleServerTestCase.test_introspection2", test_xmlrpc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SimpleServerTestCase.test_introspection2 did not pass"
print("SimpleServerTestCase::test_introspection2: ok")
