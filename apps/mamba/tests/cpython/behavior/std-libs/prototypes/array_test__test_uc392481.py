# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "prototypes"
# dimension = "behavior"
# case = "array_test__test_uc392481"
# subject = "cpython.test_prototypes.ArrayTest.test"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_prototypes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_prototypes
_suite = unittest.defaultTestLoader.loadTestsFromName("ArrayTest.test", test_prototypes)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ArrayTest.test did not pass"
print("ArrayTest::test: ok")
