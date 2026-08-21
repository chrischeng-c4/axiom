# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bigaddrspace"
# dimension = "behavior"
# case = "bytes_test__test_optimized_concat_uc5ef8bb"
# subject = "cpython.test_bigaddrspace.BytesTest.test_optimized_concat"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bigaddrspace.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bigaddrspace
_suite = unittest.defaultTestLoader.loadTestsFromName("BytesTest.test_optimized_concat", test_bigaddrspace)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BytesTest.test_optimized_concat did not pass"
print("BytesTest::test_optimized_concat: ok")
