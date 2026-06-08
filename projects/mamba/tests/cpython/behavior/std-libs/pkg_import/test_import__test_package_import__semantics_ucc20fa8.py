# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkg_import"
# dimension = "behavior"
# case = "test_import__test_package_import__semantics_ucc20fa8"
# subject = "cpython.test_pkg_import.TestImport.test_package_import__semantics"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_pkg_import.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_pkg_import
_suite = unittest.defaultTestLoader.loadTestsFromName("TestImport.test_package_import__semantics", test_pkg_import)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestImport.test_package_import__semantics did not pass"
print("TestImport::test_package_import__semantics: ok")
