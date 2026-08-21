# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "test_gzip__test_readlines_uc2b7fda"
# subject = "cpython.test_gzip.TestGzip.test_readlines"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_gzip
_suite = unittest.defaultTestLoader.loadTestsFromName("TestGzip.test_readlines", test_gzip)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestGzip.test_readlines did not pass"
print("TestGzip::test_readlines: ok")
