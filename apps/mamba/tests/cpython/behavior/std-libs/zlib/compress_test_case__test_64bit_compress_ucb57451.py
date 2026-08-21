# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_test_case__test_64bit_compress_ucb57451"
# subject = "cpython.test_zlib.CompressTestCase.test_64bit_compress"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_zlib
_suite = unittest.defaultTestLoader.loadTestsFromName("CompressTestCase.test_64bit_compress", test_zlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CompressTestCase.test_64bit_compress did not pass"
print("CompressTestCase::test_64bit_compress: ok")
