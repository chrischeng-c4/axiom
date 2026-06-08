# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "zlib_decompressor_test__test_refleaks_in___init___uc69ffa9"
# subject = "cpython.test_zlib.ZlibDecompressorTest.test_refleaks_in___init__"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_zlib
_suite = unittest.defaultTestLoader.loadTestsFromName("ZlibDecompressorTest.test_refleaks_in___init__", test_zlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ZlibDecompressorTest.test_refleaks_in___init__ did not pass"
print("ZlibDecompressorTest::test_refleaks_in___init__: ok")
