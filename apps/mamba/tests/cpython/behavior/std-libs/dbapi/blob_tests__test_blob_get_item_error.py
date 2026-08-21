# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "blob_tests__test_blob_get_item_error"
# subject = "cpython.test_dbapi.BlobTests.test_blob_get_item_error"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_dbapi.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_dbapi
_suite = unittest.defaultTestLoader.loadTestsFromName("BlobTests.test_blob_get_item_error", test_dbapi)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BlobTests.test_blob_get_item_error did not pass"
print("BlobTests::test_blob_get_item_error: ok")
