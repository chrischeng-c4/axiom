# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "site"
# dimension = "behavior"
# case = "startup_import_tests__test_startup_imports_ucf0ba89"
# subject = "cpython.test_site.StartupImportTests.test_startup_imports"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_site.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_site
_suite = unittest.defaultTestLoader.loadTestsFromName("StartupImportTests.test_startup_imports", test_site)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StartupImportTests.test_startup_imports did not pass"
print("StartupImportTests::test_startup_imports: ok")
