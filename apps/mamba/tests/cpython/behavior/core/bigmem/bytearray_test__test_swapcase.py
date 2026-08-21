# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "bigmem"
# dimension = "behavior"
# case = "bytearray_test__test_swapcase"
# subject = "cpython.test_bigmem.BytearrayTest.test_swapcase"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bigmem.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bigmem
_suite = unittest.defaultTestLoader.loadTestsFromName("BytearrayTest.test_swapcase", test_bigmem)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BytearrayTest.test_swapcase did not pass"
print("BytearrayTest::test_swapcase: ok")
