# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "sqlite_type_tests__test_too_large_int"
# subject = "cpython.test_types.SqliteTypeTests.test_too_large_int"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_types
_suite = unittest.defaultTestLoader.loadTestsFromName("SqliteTypeTests.test_too_large_int", test_types)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SqliteTypeTests.test_too_large_int did not pass"
print("SqliteTypeTests::test_too_large_int: ok")
