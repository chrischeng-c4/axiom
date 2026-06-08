# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "byte_array_test__test_init_alloc"
# subject = "cpython.test_bytes.ByteArrayTest.test_init_alloc"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bytes
_suite = unittest.defaultTestLoader.loadTestsFromName("ByteArrayTest.test_init_alloc", test_bytes)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ByteArrayTest.test_init_alloc did not pass"
print("ByteArrayTest::test_init_alloc: ok")
