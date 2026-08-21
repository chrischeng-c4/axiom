# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "buffers"
# dimension = "behavior"
# case = "string_buffer_test_case__test_create_unicode_buffer_non_bmp_uca11fb9"
# subject = "cpython.test_buffers.StringBufferTestCase.test_create_unicode_buffer_non_bmp"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_buffers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_buffers
_suite = unittest.defaultTestLoader.loadTestsFromName("StringBufferTestCase.test_create_unicode_buffer_non_bmp", test_buffers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StringBufferTestCase.test_create_unicode_buffer_non_bmp did not pass"
print("StringBufferTestCase::test_create_unicode_buffer_non_bmp: ok")
