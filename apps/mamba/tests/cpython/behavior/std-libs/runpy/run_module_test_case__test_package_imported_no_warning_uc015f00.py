# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "runpy"
# dimension = "behavior"
# case = "run_module_test_case__test_package_imported_no_warning_uc015f00"
# subject = "cpython.test_runpy.RunModuleTestCase.test_package_imported_no_warning"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_runpy.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_runpy
_suite = unittest.defaultTestLoader.loadTestsFromName("RunModuleTestCase.test_package_imported_no_warning", test_runpy)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RunModuleTestCase.test_package_imported_no_warning did not pass"
print("RunModuleTestCase::test_package_imported_no_warning: ok")
