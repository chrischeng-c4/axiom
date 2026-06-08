# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "misc_test__test_disallow_instantiation"
# subject = "cpython.test_array.MiscTest.test_disallow_instantiation"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_array
_suite = unittest.defaultTestLoader.loadTestsFromName("MiscTest.test_disallow_instantiation", test_array)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MiscTest.test_disallow_instantiation did not pass"
print("MiscTest::test_disallow_instantiation: ok")
