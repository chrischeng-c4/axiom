# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "sqlite_type_tests__test_string_with_surrogates"
# subject = "cpython.test_types.SqliteTypeTests.test_string_with_surrogates"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_types
_suite = unittest.defaultTestLoader.loadTestsFromName("SqliteTypeTests.test_string_with_surrogates", test_types)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SqliteTypeTests.test_string_with_surrogates did not pass"
print("SqliteTypeTests::test_string_with_surrogates: ok")
