# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "buffers"
# dimension = "behavior"
# case = "string_buffer_test_case__test_unicode_buffer_ucf49db8"
# subject = "cpython.test_buffers.StringBufferTestCase.test_unicode_buffer"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_buffers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_buffers
_suite = unittest.defaultTestLoader.loadTestsFromName("StringBufferTestCase.test_unicode_buffer", test_buffers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StringBufferTestCase.test_unicode_buffer did not pass"
print("StringBufferTestCase::test_unicode_buffer: ok")
