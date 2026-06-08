# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "factory"
# dimension = "behavior"
# case = "row_factory_tests__test_sqlite_row_index_uc233465"
# subject = "cpython.test_factory.RowFactoryTests.test_sqlite_row_index"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_factory.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_factory
_suite = unittest.defaultTestLoader.loadTestsFromName("RowFactoryTests.test_sqlite_row_index", test_factory)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RowFactoryTests.test_sqlite_row_index did not pass"
print("RowFactoryTests::test_sqlite_row_index: ok")
