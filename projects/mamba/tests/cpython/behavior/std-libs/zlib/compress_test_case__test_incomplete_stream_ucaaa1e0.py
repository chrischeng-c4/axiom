# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_test_case__test_incomplete_stream_ucaaa1e0"
# subject = "cpython.test_zlib.CompressTestCase.test_incomplete_stream"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_zlib
_suite = unittest.defaultTestLoader.loadTestsFromName("CompressTestCase.test_incomplete_stream", test_zlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CompressTestCase.test_incomplete_stream did not pass"
print("CompressTestCase::test_incomplete_stream: ok")
