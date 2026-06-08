# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "behavior"
# case = "pkgutil_tests__test_name_resolution_import_rebinding_ucf61bde"
# subject = "cpython.test_pkgutil.PkgutilTests.test_name_resolution_import_rebinding"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pkgutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pkgutil
_suite = unittest.defaultTestLoader.loadTestsFromName("PkgutilTests.test_name_resolution_import_rebinding", test_pkgutil)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PkgutilTests.test_name_resolution_import_rebinding did not pass"
print("PkgutilTests::test_name_resolution_import_rebinding: ok")
