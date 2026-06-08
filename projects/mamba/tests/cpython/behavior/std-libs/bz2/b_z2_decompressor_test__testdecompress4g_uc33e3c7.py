# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "b_z2_decompressor_test__testdecompress4g_uc33e3c7"
# subject = "cpython.test_bz2.BZ2DecompressorTest.testDecompress4G"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bz2
_suite = unittest.defaultTestLoader.loadTestsFromName("BZ2DecompressorTest.testDecompress4G", test_bz2)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BZ2DecompressorTest.testDecompress4G did not pass"
print("BZ2DecompressorTest::testDecompress4G: ok")
