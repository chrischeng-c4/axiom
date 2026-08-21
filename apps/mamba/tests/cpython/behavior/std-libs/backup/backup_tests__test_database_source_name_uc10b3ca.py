# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "backup"
# dimension = "behavior"
# case = "backup_tests__test_database_source_name_uc10b3ca"
# subject = "cpython.test_backup.BackupTests.test_database_source_name"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_backup.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_backup
_suite = unittest.defaultTestLoader.loadTestsFromName("BackupTests.test_database_source_name", test_backup)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BackupTests.test_database_source_name did not pass"
print("BackupTests::test_database_source_name: ok")
