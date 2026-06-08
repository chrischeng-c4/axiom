# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "gzip_stream_write_test__test_source_directory_not_leaked"
# subject = "cpython.test_tarfile.GzipStreamWriteTest.test_source_directory_not_leaked"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tarfile
_suite = unittest.defaultTestLoader.loadTestsFromName("GzipStreamWriteTest.test_source_directory_not_leaked", test_tarfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GzipStreamWriteTest.test_source_directory_not_leaked did not pass"
print("GzipStreamWriteTest::test_source_directory_not_leaked: ok")
