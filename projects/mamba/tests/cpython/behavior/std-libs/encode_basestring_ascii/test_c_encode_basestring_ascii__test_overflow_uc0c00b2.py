# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encode_basestring_ascii"
# dimension = "behavior"
# case = "test_c_encode_basestring_ascii__test_overflow_uc0c00b2"
# subject = "cpython.test_encode_basestring_ascii.TestCEncodeBasestringAscii.test_overflow"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_json/test_encode_basestring_ascii.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_json import test_encode_basestring_ascii
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCEncodeBasestringAscii.test_overflow", test_encode_basestring_ascii)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCEncodeBasestringAscii.test_overflow did not pass"
print("TestCEncodeBasestringAscii::test_overflow: ok")
