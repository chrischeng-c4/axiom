# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "open_tests__test_factory_database_arg"
# subject = "cpython.test_dbapi.OpenTests.test_factory_database_arg"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_dbapi.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_dbapi
_suite = unittest.defaultTestLoader.loadTestsFromName("OpenTests.test_factory_database_arg", test_dbapi)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython OpenTests.test_factory_database_arg did not pass"
print("OpenTests::test_factory_database_arg: ok")
