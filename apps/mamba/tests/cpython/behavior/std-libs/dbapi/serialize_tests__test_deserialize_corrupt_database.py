# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "serialize_tests__test_deserialize_corrupt_database"
# subject = "cpython.test_dbapi.SerializeTests.test_deserialize_corrupt_database"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_dbapi.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_dbapi
_suite = unittest.defaultTestLoader.loadTestsFromName("SerializeTests.test_deserialize_corrupt_database", test_dbapi)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SerializeTests.test_deserialize_corrupt_database did not pass"
print("SerializeTests::test_deserialize_corrupt_database: ok")
