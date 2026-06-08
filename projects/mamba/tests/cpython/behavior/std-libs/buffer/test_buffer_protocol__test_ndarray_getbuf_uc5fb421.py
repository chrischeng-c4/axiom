# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "buffer"
# dimension = "behavior"
# case = "test_buffer_protocol__test_ndarray_getbuf_uc5fb421"
# subject = "cpython.test_buffer.TestBufferProtocol.test_ndarray_getbuf"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_buffer.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_buffer
_suite = unittest.defaultTestLoader.loadTestsFromName("TestBufferProtocol.test_ndarray_getbuf", test_buffer)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestBufferProtocol.test_ndarray_getbuf did not pass"
print("TestBufferProtocol::test_ndarray_getbuf: ok")
