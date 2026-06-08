# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "userfunctions"
# dimension = "behavior"
# case = "authorizer_tests__test_column_access"
# subject = "cpython.test_userfunctions.AuthorizerTests.test_column_access"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_userfunctions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_userfunctions
_suite = unittest.defaultTestLoader.loadTestsFromName("AuthorizerTests.test_column_access", test_userfunctions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AuthorizerTests.test_column_access did not pass"
print("AuthorizerTests::test_column_access: ok")
