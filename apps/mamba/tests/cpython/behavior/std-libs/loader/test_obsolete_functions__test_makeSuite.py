# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "loader"
# dimension = "behavior"
# case = "test_obsolete_functions__test_makeSuite"
# subject = "cpython.test_loader.TestObsoleteFunctions.test_makeSuite"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_loader.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_loader
_suite = unittest.defaultTestLoader.loadTestsFromName("TestObsoleteFunctions.test_makeSuite", test_loader)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestObsoleteFunctions.test_makeSuite did not pass"
print("TestObsoleteFunctions::test_makeSuite: ok")
