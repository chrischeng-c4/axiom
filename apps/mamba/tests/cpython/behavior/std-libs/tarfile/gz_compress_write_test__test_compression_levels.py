# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "gz_compress_write_test__test_compression_levels"
# subject = "cpython.test_tarfile.GzCompressWriteTest.test_compression_levels"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tarfile
_suite = unittest.defaultTestLoader.loadTestsFromName("GzCompressWriteTest.test_compression_levels", test_tarfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GzCompressWriteTest.test_compression_levels did not pass"
print("GzCompressWriteTest::test_compression_levels: ok")
