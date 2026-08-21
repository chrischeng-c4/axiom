# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dump"
# dimension = "behavior"
# case = "dump_tests__test_dump_autoincrement_create_new_db_uc67d5f1"
# subject = "cpython.test_dump.DumpTests.test_dump_autoincrement_create_new_db"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_dump.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_dump
_suite = unittest.defaultTestLoader.loadTestsFromName("DumpTests.test_dump_autoincrement_create_new_db", test_dump)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DumpTests.test_dump_autoincrement_create_new_db did not pass"
print("DumpTests::test_dump_autoincrement_create_new_db: ok")
