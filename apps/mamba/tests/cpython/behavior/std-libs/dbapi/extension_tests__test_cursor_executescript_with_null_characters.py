# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "extension_tests__test_cursor_executescript_with_null_characters"
# subject = "cpython.test_dbapi.ExtensionTests.test_cursor_executescript_with_null_characters"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_dbapi.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_dbapi
_suite = unittest.defaultTestLoader.loadTestsFromName("ExtensionTests.test_cursor_executescript_with_null_characters", test_dbapi)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ExtensionTests.test_cursor_executescript_with_null_characters did not pass"
print("ExtensionTests::test_cursor_executescript_with_null_characters: ok")
