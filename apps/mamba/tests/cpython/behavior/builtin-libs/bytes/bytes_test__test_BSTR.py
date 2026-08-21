# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bytes"
# dimension = "behavior"
# case = "bytes_test__test_BSTR"
# subject = "cpython.test_bytes.BytesTest.test_BSTR"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_bytes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_bytes
_suite = unittest.defaultTestLoader.loadTestsFromName("BytesTest.test_BSTR", test_bytes)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BytesTest.test_BSTR did not pass"
print("BytesTest::test_BSTR: ok")
