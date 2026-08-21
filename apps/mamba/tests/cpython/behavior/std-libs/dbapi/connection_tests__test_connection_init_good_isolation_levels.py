# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbapi"
# dimension = "behavior"
# case = "connection_tests__test_connection_init_good_isolation_levels"
# subject = "cpython.test_dbapi.ConnectionTests.test_connection_init_good_isolation_levels"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_dbapi.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_dbapi
_suite = unittest.defaultTestLoader.loadTestsFromName("ConnectionTests.test_connection_init_good_isolation_levels", test_dbapi)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ConnectionTests.test_connection_init_good_isolation_levels did not pass"
print("ConnectionTests::test_connection_init_good_isolation_levels: ok")
