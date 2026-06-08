# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc"
# dimension = "behavior"
# case = "use_builtin_types_test_case__test_use_builtin_types"
# subject = "cpython.test_xmlrpc.UseBuiltinTypesTestCase.test_use_builtin_types"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xmlrpc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xmlrpc
_suite = unittest.defaultTestLoader.loadTestsFromName("UseBuiltinTypesTestCase.test_use_builtin_types", test_xmlrpc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UseBuiltinTypesTestCase.test_use_builtin_types did not pass"
print("UseBuiltinTypesTestCase::test_use_builtin_types: ok")
