# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "assorted_bytes_test__test_compare"
# subject = "cpython.test_bytes.AssortedBytesTest.test_compare"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bytes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bytes
_suite = unittest.defaultTestLoader.loadTestsFromName("AssortedBytesTest.test_compare", test_bytes)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AssortedBytesTest.test_compare did not pass"
print("AssortedBytesTest::test_compare: ok")
