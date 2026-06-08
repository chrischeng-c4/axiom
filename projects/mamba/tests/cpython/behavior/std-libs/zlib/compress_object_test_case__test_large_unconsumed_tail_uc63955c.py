# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compress_object_test_case__test_large_unconsumed_tail_uc63955c"
# subject = "cpython.test_zlib.CompressObjectTestCase.test_large_unconsumed_tail"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_zlib
_suite = unittest.defaultTestLoader.loadTestsFromName("CompressObjectTestCase.test_large_unconsumed_tail", test_zlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CompressObjectTestCase.test_large_unconsumed_tail did not pass"
print("CompressObjectTestCase::test_large_unconsumed_tail: ok")
