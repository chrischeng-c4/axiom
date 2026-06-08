# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "compressor_decompressor_test_case__test_decompressor_bigmem_uc438dcb"
# subject = "cpython.test_lzma.CompressorDecompressorTestCase.test_decompressor_bigmem"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_lzma
_suite = unittest.defaultTestLoader.loadTestsFromName("CompressorDecompressorTestCase.test_decompressor_bigmem", test_lzma)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CompressorDecompressorTestCase.test_decompressor_bigmem did not pass"
print("CompressorDecompressorTestCase::test_decompressor_bigmem: ok")
