# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "compressor_decompressor_test_case__test_refleaks_in_decompressor___init___uc28434f"
# subject = "cpython.test_lzma.CompressorDecompressorTestCase.test_refleaks_in_decompressor___init__"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_lzma
_suite = unittest.defaultTestLoader.loadTestsFromName("CompressorDecompressorTestCase.test_refleaks_in_decompressor___init__", test_lzma)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CompressorDecompressorTestCase.test_refleaks_in_decompressor___init__ did not pass"
print("CompressorDecompressorTestCase::test_refleaks_in_decompressor___init__: ok")
