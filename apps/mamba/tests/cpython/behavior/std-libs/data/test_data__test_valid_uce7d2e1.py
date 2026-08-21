# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "data"
# dimension = "behavior"
# case = "test_data__test_valid_uce7d2e1"
# subject = "cpython.test_data.TestData.test_valid"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_data.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_tomllib import test_data
_suite = unittest.defaultTestLoader.loadTestsFromName("TestData.test_valid", test_data)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestData.test_valid did not pass"
print("TestData::test_valid: ok")
