# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "decl_types_tests__test_blob"
# subject = "cpython.test_types.DeclTypesTests.test_blob"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_types
_suite = unittest.defaultTestLoader.loadTestsFromName("DeclTypesTests.test_blob", test_types)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DeclTypesTests.test_blob did not pass"
print("DeclTypesTests::test_blob: ok")
