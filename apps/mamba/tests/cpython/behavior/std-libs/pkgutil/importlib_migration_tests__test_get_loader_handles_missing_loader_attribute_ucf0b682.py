# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "behavior"
# case = "importlib_migration_tests__test_get_loader_handles_missing_loader_attribute_ucf0b682"
# subject = "cpython.test_pkgutil.ImportlibMigrationTests.test_get_loader_handles_missing_loader_attribute"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pkgutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pkgutil
_suite = unittest.defaultTestLoader.loadTestsFromName("ImportlibMigrationTests.test_get_loader_handles_missing_loader_attribute", test_pkgutil)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ImportlibMigrationTests.test_get_loader_handles_missing_loader_attribute did not pass"
print("ImportlibMigrationTests::test_get_loader_handles_missing_loader_attribute: ok")
