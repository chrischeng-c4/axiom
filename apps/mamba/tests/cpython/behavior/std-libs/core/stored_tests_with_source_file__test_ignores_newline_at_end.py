# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "stored_tests_with_source_file__test_ignores_newline_at_end"
# subject = "cpython.test_core.StoredTestsWithSourceFile.test_ignores_newline_at_end"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipfile/test_core.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zipfile import test_core
_suite = unittest.defaultTestLoader.loadTestsFromName("StoredTestsWithSourceFile.test_ignores_newline_at_end", test_core)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StoredTestsWithSourceFile.test_ignores_newline_at_end did not pass"
print("StoredTestsWithSourceFile::test_ignores_newline_at_end: ok")
