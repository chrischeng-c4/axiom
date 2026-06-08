# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "zlib_decompressor_test__testdecompress4g_uc9c274c"
# subject = "cpython.test_zlib.ZlibDecompressorTest.testDecompress4G"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_zlib
_suite = unittest.defaultTestLoader.loadTestsFromName("ZlibDecompressorTest.testDecompress4G", test_zlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ZlibDecompressorTest.testDecompress4G did not pass"
print("ZlibDecompressorTest::testDecompress4G: ok")
