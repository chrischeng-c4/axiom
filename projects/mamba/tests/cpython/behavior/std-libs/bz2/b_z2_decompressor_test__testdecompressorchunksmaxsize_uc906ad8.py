# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "b_z2_decompressor_test__testdecompressorchunksmaxsize_uc906ad8"
# subject = "cpython.test_bz2.BZ2DecompressorTest.testDecompressorChunksMaxsize"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bz2
_suite = unittest.defaultTestLoader.loadTestsFromName("BZ2DecompressorTest.testDecompressorChunksMaxsize", test_bz2)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BZ2DecompressorTest.testDecompressorChunksMaxsize did not pass"
print("BZ2DecompressorTest::testDecompressorChunksMaxsize: ok")
