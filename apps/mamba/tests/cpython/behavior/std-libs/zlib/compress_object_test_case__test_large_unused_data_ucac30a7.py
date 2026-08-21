# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_large_unused_data_ucac30a7"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_large_unused_data"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_zlib
_suite = unittest.defaultTestLoader.loadTestsFromName("CompressObjectTestCase.test_large_unused_data", test_zlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CompressObjectTestCase.test_large_unused_data did not pass"
print("CompressObjectTestCase::test_large_unused_data: ok")
