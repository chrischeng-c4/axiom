# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "buffer"
# dimension = "behavior"
# case = "test_buffer_protocol__test_ndarray_random_slice_assign_ucc9b7c9"
# subject = "cpython.test_buffer.TestBufferProtocol.test_ndarray_random_slice_assign"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_buffer.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_buffer
_suite = unittest.defaultTestLoader.loadTestsFromName("TestBufferProtocol.test_ndarray_random_slice_assign", test_buffer)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestBufferProtocol.test_ndarray_random_slice_assign did not pass"
print("TestBufferProtocol::test_ndarray_random_slice_assign: ok")
