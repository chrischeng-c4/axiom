# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "deflate_tests_with_source_file__test_per_file_compression"
# subject = "cpython.test_core.DeflateTestsWithSourceFile.test_per_file_compression"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipfile/test_core.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zipfile import test_core
_suite = unittest.defaultTestLoader.loadTestsFromName("DeflateTestsWithSourceFile.test_per_file_compression", test_core)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DeflateTestsWithSourceFile.test_per_file_compression did not pass"
print("DeflateTestsWithSourceFile::test_per_file_compression: ok")
