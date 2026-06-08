# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "test_gzip__test_seek_read_uc98036c"
# subject = "cpython.test_gzip.TestGzip.test_seek_read"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_gzip
_suite = unittest.defaultTestLoader.loadTestsFromName("TestGzip.test_seek_read", test_gzip)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestGzip.test_seek_read did not pass"
print("TestGzip::test_seek_read: ok")
