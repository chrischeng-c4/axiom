# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "loader"
# dimension = "behavior"
# case = "test__test_loader__test_loadTestsFromName__empty_name"
# subject = "cpython.test_loader.Test_TestLoader.test_loadTestsFromName__empty_name"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_loader.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_loader
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_TestLoader.test_loadTestsFromName__empty_name", test_loader)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_TestLoader.test_loadTestsFromName__empty_name did not pass"
print("Test_TestLoader::test_loadTestsFromName__empty_name: ok")
