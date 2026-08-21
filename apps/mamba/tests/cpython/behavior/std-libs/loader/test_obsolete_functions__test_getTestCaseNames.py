# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "loader"
# dimension = "behavior"
# case = "test_obsolete_functions__test_getTestCaseNames"
# subject = "cpython.test_loader.TestObsoleteFunctions.test_getTestCaseNames"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_loader.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_loader
_suite = unittest.defaultTestLoader.loadTestsFromName("TestObsoleteFunctions.test_getTestCaseNames", test_loader)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestObsoleteFunctions.test_getTestCaseNames did not pass"
print("TestObsoleteFunctions::test_getTestCaseNames: ok")
