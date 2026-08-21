# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_mutating_index"
# subject = "cpython.test_bytes.ByteArrayTest.test_mutating_index"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bytes
_suite = unittest.defaultTestLoader.loadTestsFromName("ByteArrayTest.test_mutating_index", test_bytes)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ByteArrayTest.test_mutating_index did not pass"
print("ByteArrayTest::test_mutating_index: ok")
