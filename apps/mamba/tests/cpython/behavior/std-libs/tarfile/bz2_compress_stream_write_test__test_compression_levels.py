# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "bz2_compress_stream_write_test__test_compression_levels"
# subject = "cpython.test_tarfile.Bz2CompressStreamWriteTest.test_compression_levels"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tarfile
_suite = unittest.defaultTestLoader.loadTestsFromName("Bz2CompressStreamWriteTest.test_compression_levels", test_tarfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Bz2CompressStreamWriteTest.test_compression_levels did not pass"
print("Bz2CompressStreamWriteTest::test_compression_levels: ok")
