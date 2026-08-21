# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "test_open__test_pathlike_file_uc638fba"
# subject = "cpython.test_gzip.TestOpen.test_pathlike_file"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_gzip
_suite = unittest.defaultTestLoader.loadTestsFromName("TestOpen.test_pathlike_file", test_gzip)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestOpen.test_pathlike_file did not pass"
print("TestOpen::test_pathlike_file: ok")
